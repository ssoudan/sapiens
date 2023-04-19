use std::rc::Rc;

use indoc::indoc;
use sapiens::invoke_tool;
use sapiens::tools::Toolbox;
use sapiens_tools::dummy::DummyTool;
use sapiens_tools::python;
use sapiens_tools::python::PythonTool;

#[test]
fn test_tool_invocation() {
    let data = indoc! {r#"
        # Action
        ```yaml        
        command: SandboxedPython
        input:
            code: |
                print("Hello world!")          
        ```
        "#};

    let mut toolbox = Toolbox::default();
    toolbox.add_advanced_tool(PythonTool::default());

    let toolbox = Rc::new(toolbox);

    let output = invoke_tool(toolbox, data).unwrap();
    assert_eq!(output, "stdout: |\n  Hello world!\nstderr: ''\n");
}

#[test]
fn test_tool_invocation_in_python() {
    let data = indoc! {r#"
        # Action
        ```yaml        
        command: SandboxedPython
        input:
            code: |
                print("Hello world!")
                rooms = toolbox.invoke("Dummy", {"blah": "blah"})
                print(rooms)
                rooms = tools.dummy(blah="blah")
                print(rooms)          
        ```
        "#};

    let mut toolbox = Toolbox::default();
    toolbox.add_advanced_tool(PythonTool::default());
    toolbox.add_tool(DummyTool::default());

    let toolbox = Rc::new(toolbox);

    let output = invoke_tool(toolbox, data).unwrap();
    assert_eq!(
            output,
            "stdout: |\n  Hello world!\n  {'something': 'blah and something else'}\n  {'something': 'blah and something else'}\nstderr: ''\n"
        );
}

#[test]
fn test_multiple_tool_invocations() {
    let data = indoc! {r#"
        # Action
        ```yaml        
        command: SandboxedPython
        input:
            code: |
                print("Hello world 1!")          
        ```
        
        # And another action
        ```yaml        
        command: SandboxedPython
        input:
            code: |
                print("Hello world 2!")          
        ```
        
        # And yet another action
        ```        
        command: SandboxedPython
        input:
            code: |
                print("Hello world 3!")          
        ```
        "#};

    let mut toolbox = Toolbox::default();
    toolbox.add_advanced_tool(PythonTool::default());

    let toolbox = Rc::new(toolbox);

    let output = invoke_tool(toolbox, data).unwrap();
    assert_eq!(output, "stdout: |\n  Hello world 1!\nstderr: ''\n");
}

#[test]
fn test_python() {
    let mut toolbox = Toolbox::default();
    toolbox.add_tool(DummyTool::default());
    toolbox.add_advanced_tool(python::PythonTool::default());

    let toolbox = Rc::new(toolbox);

    let input = indoc! {
    r#"```yaml
       command: SandboxedPython
       input:
         code: |           
           args = {
               'blah': "hello"
           }
           output = tools.Dummy(**args)           
          
           something = output['something']                       

           print(f"And the result is: {something}")
       ```
    "#};

    let res = invoke_tool(toolbox, input).unwrap();

    assert_eq!(
        res,
        "stdout: |\n  And the result is: hello and something else\nstderr: ''\n"
    );
}

#[cfg(feature = "wiki")]
mod wiki {
    use indoc::indoc;

    #[test]
    fn test_wikidata_sparql() {
        let query = indoc! {
r#"
            PREFIX wd: <http://www.wikidata.org/entity/>
            PREFIX wdt: <http://www.wikidata.org/prop/direct/>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            
            SELECT ?country ?countryLabel ?capital ?capitalLabel
            WHERE {
              ?country wdt:P31 wd:Q6256 .         # ?country is an instance of a country (Q6256)
              ?country wdt:P36 ?capital .          # ?country has a capital (?capital)
              SERVICE wikibase:label {
                bd:serviceParam wikibase:language "en" .    # Use English labels
                ?country rdfs:label ?countryLabel .
                ?capital rdfs:label ?capitalLabel .
              }
            }
            ORDER BY ?countryLabel
            LIMIT 10
        "#
        };

        let api = mediawiki::api_sync::ApiSync::new("https://www.wikidata.org/w/api.php").unwrap(); // Will determine the SPARQL API URL via site info data
        let res = api.sparql_query(query).unwrap();
        println!("{}", serde_json::to_string_pretty(&res).unwrap());
    }

    #[test]
    fn test_wikipedia() {
        let api = mediawiki::api_sync::ApiSync::new("https://en.wikipedia.org/w/api.php").unwrap();

        // Query parameters
        let params = api.params_into(&[
            ("action", "query"),
            ("prop", "extracts|explaintext|exintro"),
            ("titles", "Albert Einstein"),
        ]);

        let res = api.get_query_api_json_all(&params).unwrap();

        // Print the result
        println!("{}", serde_json::to_string_pretty(&res).unwrap());
    }
}
