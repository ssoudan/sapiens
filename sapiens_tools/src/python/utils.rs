use pyo3::types::{IntoPyDict, PyDict, PyFloat, PyList, PyTuple};
use pyo3::{PyObject, Python, ToPyObject};
use sapiens::tools::{FieldFormat, ToolDescription};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(thiserror::Error, Debug)]
pub(crate) enum PyConversionError {
    #[error("Invalid conversion: {error}")]
    InvalidConversion { error: String },
    #[error("dict key not serializable: {typename}")]
    DictKeyNotSerializable { typename: String },
    #[error("Invalid cast: {typename}")]
    InvalidCast { typename: String },
}

// inspired from https://github.com/mozilla-services/python-canonicaljson-rs/blob/62599b246055a1c8a78e5777acdfe0fd594be3d8/src/lib.rs#L87-L167
pub(crate) fn to_yaml(py: Python, obj: &PyObject) -> Result<Value, PyConversionError> {
    macro_rules! return_cast {
        ($t:ty, $f:expr) => {
            if let Ok(val) = obj.downcast::<$t>(py) {
                return $f(val);
            }
        };
    }

    macro_rules! return_to_value {
        ($t:ty) => {
            if let Ok(val) = obj.extract::<$t>(py) {
                return serde_yaml::to_value(val).map_err(|error| {
                    PyConversionError::InvalidConversion {
                        error: format!("{}", error),
                    }
                });
            }
        };
    }

    if obj.is_none(py) {
        return Ok(Value::Null);
    }

    return_to_value!(String);
    return_to_value!(bool);
    return_to_value!(u64);
    return_to_value!(i64);

    return_cast!(PyDict, |x: &PyDict| {
        let mut map = serde_yaml::Mapping::new();
        for (key_obj, value) in x.iter() {
            let key = if key_obj.is_none() {
                Ok("null".to_string())
            } else if let Ok(val) = key_obj.extract::<bool>() {
                Ok(if val {
                    "true".to_string()
                } else {
                    "false".to_string()
                })
            } else if let Ok(val) = key_obj.str() {
                Ok(val.to_string())
            } else {
                Err(PyConversionError::DictKeyNotSerializable {
                    typename: key_obj
                        .to_object(py)
                        .as_ref(py)
                        .get_type()
                        .name()
                        .map(|x| x.to_string())
                        .unwrap_or_else(|_| "unknown".to_string()),
                })
            };
            map.insert(Value::String(key?), to_yaml(py, &value.to_object(py))?);
        }
        Ok(Value::Mapping(map))
    });

    return_cast!(PyList, |x: &PyList| {
        let v = x
            .iter()
            .map(|x| to_yaml(py, &x.to_object(py)))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::Sequence(v))
    });

    return_cast!(PyTuple, |x: &PyTuple| {
        let v = x
            .iter()
            .map(|x| to_yaml(py, &x.to_object(py)))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::Sequence(v))
    });

    return_cast!(PyFloat, |x: &PyFloat| {
        Ok(Value::Number(serde_yaml::Number::from(x.value())))
    });

    // At this point we can't cast it, set up the error object
    Err(PyConversionError::InvalidCast {
        typename: obj
            .as_ref(py)
            .get_type()
            .name()
            .map(|x| x.to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
    })
}

pub(crate) fn value_to_object(val: Value, py: Python<'_>) -> PyObject {
    match val {
        Value::Null => py.None(),
        Value::Bool(x) => x.to_object(py),
        Value::Number(x) => {
            let oi64 = x.as_i64().map(|i| i.to_object(py));
            let ou64 = x.as_u64().map(|i| i.to_object(py));
            let of64 = x.as_f64().map(|i| i.to_object(py));
            oi64.or(ou64).or(of64).expect("number too large")
        }
        Value::String(x) => x.to_object(py),
        Value::Sequence(x) => {
            let inner: Vec<_> = x.into_iter().map(|x| value_to_object(x, py)).collect();
            inner.to_object(py)
        }
        Value::Mapping(x) => {
            let iter = x
                .into_iter()
                .map(|(k, v)| (value_to_object(k, py), value_to_object(v, py)));
            IntoPyDict::into_py_dict(iter, py).into()
        }
        Value::Tagged(_) => panic!("tagged values are not supported"),
    }
}

/// Format of a field in a tool description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleFormat {
    /// Name of the field
    pub name: String,
    /// Type of the field
    pub r#type: String,
    /// Whether the field is optional
    pub optional: bool,
    /// Description of the field
    pub description: String,
}

impl From<FieldFormat> for SimpleFormat {
    fn from(part: FieldFormat) -> Self {
        SimpleFormat {
            name: part.name,
            r#type: part.r#type,
            optional: part.optional,
            description: part.description,
        }
    }
}

impl ToPyObject for SimpleFormat {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let dict = PyDict::new(py);
        dict.set_item("name", self.name.to_object(py)).unwrap();
        dict.set_item("type", self.r#type.to_object(py)).unwrap();
        dict.set_item("optional", self.optional.to_object(py))
            .unwrap();
        dict.set_item("description", self.description.to_object(py))
            .unwrap();
        dict.into()
    }
}

/// A simplified version of the tool description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SimpleToolDescription {
    /// Name of the tool
    pub name: String,
    /// Description of the tool
    pub description: String,
    /// Input format of the tool
    pub input_format: Vec<SimpleFormat>,
    /// Output format of the tool
    pub output_format: Vec<SimpleFormat>,
}

impl From<ToolDescription> for SimpleToolDescription {
    fn from(desc: ToolDescription) -> Self {
        let input_format = desc
            .input_format
            .fields
            .into_iter()
            .map(|x| x.into())
            .collect();
        let output_format = desc
            .output_format
            .fields
            .into_iter()
            .map(|x| x.into())
            .collect();

        SimpleToolDescription {
            name: desc.name,
            description: desc.description,
            input_format,
            output_format,
        }
    }
}

impl ToPyObject for SimpleToolDescription {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let dict = PyDict::new(py);
        dict.set_item("name", self.name.clone()).unwrap();
        dict.set_item("description", self.description.clone())
            .unwrap();
        dict.set_item("input_format", self.input_format.clone())
            .unwrap();
        dict.set_item("output_format", self.output_format.clone())
            .unwrap();
        dict.into()
    }
}
