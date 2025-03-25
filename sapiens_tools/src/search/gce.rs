use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Google Custom Search Engine URL
pub const URL: &str = "https://customsearch.googleapis.com/customsearch/v1";

/// Error details
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ErrorDetail {
    /// Error code
    pub code: u16,
    /// Error message
    pub message: String,
    /// Status
    pub status: String,
}

/// Error body
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ErrorBody {
    /// Error details
    pub error: ErrorDetail,
}

/// Enable Simplified/Traditional Chinese
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum C2COff {
    /// Disabled
    Off,
    /// Enable Simplified and Traditional Chinese Search
    On,
}

impl Display for C2COff {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "0"),
            Self::On => write!(f, "1"),
        }
    }
}

/// Country Restriction
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Cr {
    /// Country code - See <https://developers.google.com/custom-search/docs/json_api_reference#countryCollections>
    Country(String),
    /// Not
    Not(Box<Cr>),
    /// And
    And(Box<Cr>, Box<Cr>),
    /// Or
    Or(Box<Cr>, Box<Cr>),
}

impl Display for Cr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Country(country) => write!(f, "{country}"),
            Self::Not(cr) => write!(f, "-{cr}"),
            Self::And(cr1, cr2) => write!(f, "({cr1}).({cr2})"),
            Self::Or(cr1, cr2) => write!(f, "({cr1}|{cr2})"),
        }
    }
}

/// Date Restrict
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum DateRestrict {
    /// From the specified number of past days
    Day(u32),
    /// From the specified number of past weeks
    Week(u32),
    /// From the specified number of past months
    Month(u32),
    /// From the specified number of past years
    Year(u32),
}

impl Display for DateRestrict {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Day(day) => write!(f, "d{day}"),
            Self::Week(week) => write!(f, "w{week}"),
            Self::Month(month) => write!(f, "m{month}"),
            Self::Year(year) => write!(f, "y{year}"),
        }
    }
}

/// Duplicate Content Filter
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum DuplicateContentFilter {
    /// Off
    Off,
    /// On
    On,
}

impl Display for DuplicateContentFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "0"),
            Self::On => write!(f, "1"),
        }
    }
}

/// Geolocation of end user
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Gl {
    /// Country code - See <https://developers.google.com/custom-search/docs/json_api_reference#country-codes>
    /// (2 letters lower case).
    CountryCode(String),
}

impl Display for Gl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CountryCode(country_code) => write!(f, "{country_code}"),
        }
    }
}

/// Image Color Type
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ImgColorType {
    /// Color
    Color,
    /// Gray
    Gray,
    /// Mono
    Mono,
    /// Transparent
    Trans,
}

impl Display for ImgColorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Color => {
                write!(f, "color")
            }
            Self::Gray => {
                write!(f, "gray")
            }
            Self::Mono => {
                write!(f, "mono")
            }
            Self::Trans => {
                write!(f, "trans")
            }
        }
    }
}

/// Image dominant color
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ImgDominantColor {
    /// Black
    Black,
    /// Blue
    Blue,
    /// Brown
    Brown,
    /// Gray
    Gray,
    /// Green
    Green,
    /// Orange
    Orange,
    /// Pink
    Pink,
    /// Purple
    Purple,
    /// Red
    Red,
    /// Teal
    Teal,
    /// White
    White,
    /// Yellow
    Yellow,
}

impl Display for ImgDominantColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Black => {
                write!(f, "black")
            }
            Self::Blue => {
                write!(f, "blue")
            }
            Self::Brown => {
                write!(f, "brown")
            }
            Self::Gray => {
                write!(f, "gray")
            }
            Self::Green => {
                write!(f, "green")
            }
            Self::Orange => {
                write!(f, "orange")
            }
            Self::Pink => {
                write!(f, "pink")
            }
            Self::Purple => {
                write!(f, "purple")
            }
            Self::Red => {
                write!(f, "red")
            }
            Self::Teal => {
                write!(f, "teal")
            }
            Self::White => {
                write!(f, "white")
            }
            Self::Yellow => {
                write!(f, "yellow")
            }
        }
    }
}

/// Image Size
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ImgSize {
    /// Icon
    Icon,
    /// Small
    Small,
    /// Medium
    Medium,
    /// Large
    Large,
    /// Xlarge
    Xlarge,
    /// Xxlarge
    Xxlarge,
    /// Huge
    Huge,
}

impl Display for ImgSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Icon => {
                write!(f, "icon")
            }
            Self::Small => {
                write!(f, "small")
            }
            Self::Medium => {
                write!(f, "medium")
            }
            Self::Large => {
                write!(f, "large")
            }
            Self::Xlarge => {
                write!(f, "xlarge")
            }
            Self::Xxlarge => {
                write!(f, "xxlarge")
            }
            Self::Huge => {
                write!(f, "huge")
            }
        }
    }
}

/// Image Type
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ImgType {
    /// Face
    Face,
    /// Photo
    Photo,
    /// Clipart
    Clipart,
    /// Lineart
    Lineart,
    /// Animated
    Animated,
    /// Stock
    Stock,
}

impl Display for ImgType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Face => {
                write!(f, "face")
            }
            Self::Photo => {
                write!(f, "photo")
            }
            Self::Clipart => {
                write!(f, "clipart")
            }
            Self::Lineart => {
                write!(f, "lineart")
            }
            Self::Animated => {
                write!(f, "animated")
            }
            Self::Stock => {
                write!(f, "stock")
            }
        }
    }
}

/// Language Restriction
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Lr {
    LangAr,
    LangBg,
    LangCa,
    LangCs,
    LangDa,
    LangDe,
    LangEl,
    LangEn,
    LangEs,
    LangEt,
    LangFi,
    LangFr,
    LangHr,
    LangHu,
    LangId,
    LangIs,
    LangIt,
    LangIw,
    LangJa,
    LangKo,
    LangLt,
    LangLv,
    LangNl,
    LangNo,
    LangPl,
    LangPt,
    LangRo,
    LangRu,
    LangSk,
    LangSl,
    LangSr,
    LangSv,
    LangTr,
    LangZhCN,
    LangZhTW,
}

impl Display for Lr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LangAr => write!(f, "lang_ar"),
            Self::LangBg => write!(f, "lang_bg"),
            Self::LangCa => write!(f, "lang_ca"),
            Self::LangCs => write!(f, "lang_cs"),
            Self::LangDa => write!(f, "lang_da"),
            Self::LangDe => write!(f, "lang_de"),
            Self::LangEl => write!(f, "lang_el"),
            Self::LangEn => write!(f, "lang_en"),
            Self::LangEs => write!(f, "lang_es"),
            Self::LangEt => write!(f, "lang_et"),
            Self::LangFi => write!(f, "lang_fi"),
            Self::LangFr => write!(f, "lang_fr"),
            Self::LangHr => write!(f, "lang_hr"),
            Self::LangHu => write!(f, "lang_hu"),
            Self::LangId => write!(f, "lang_id"),
            Self::LangIs => write!(f, "lang_is"),
            Self::LangIt => write!(f, "lang_it"),
            Self::LangIw => write!(f, "lang_iw"),
            Self::LangJa => write!(f, "lang_ja"),
            Self::LangKo => write!(f, "lang_ko"),
            Self::LangLt => write!(f, "lang_lt"),
            Self::LangLv => write!(f, "lang_lv"),
            Self::LangNl => write!(f, "lang_nl"),
            Self::LangNo => write!(f, "lang_no"),
            Self::LangPl => write!(f, "lang_pl"),
            Self::LangPt => write!(f, "lang_pt"),
            Self::LangRo => write!(f, "lang_ro"),
            Self::LangRu => write!(f, "lang_ru"),
            Self::LangSk => write!(f, "lang_sk"),
            Self::LangSl => write!(f, "lang_sl"),
            Self::LangSr => write!(f, "lang_sr"),
            Self::LangSv => write!(f, "lang_sv"),
            Self::LangTr => write!(f, "lang_tr"),
            Self::LangZhCN => write!(f, "lang_zh-CN"),
            Self::LangZhTW => write!(f, "lang_zh-TW"),
        }
    }
}

impl TryFrom<&str> for Lr {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "lang_ar" => Ok(Self::LangAr),
            "lang_bg" => Ok(Self::LangBg),
            "lang_ca" => Ok(Self::LangCa),
            "lang_cs" => Ok(Self::LangCs),
            "lang_da" => Ok(Self::LangDa),
            "lang_de" => Ok(Self::LangDe),
            "lang_el" => Ok(Self::LangEl),
            "lang_en" => Ok(Self::LangEn),
            "lang_es" => Ok(Self::LangEs),
            "lang_et" => Ok(Self::LangEt),
            "lang_fi" => Ok(Self::LangFi),
            "lang_fr" => Ok(Self::LangFr),
            "lang_hr" => Ok(Self::LangHr),
            "lang_hu" => Ok(Self::LangHu),
            "lang_id" => Ok(Self::LangId),
            "lang_is" => Ok(Self::LangIs),
            "lang_it" => Ok(Self::LangIt),
            "lang_iw" => Ok(Self::LangIw),
            "lang_ja" => Ok(Self::LangJa),
            "lang_ko" => Ok(Self::LangKo),
            "lang_lt" => Ok(Self::LangLt),
            "lang_lv" => Ok(Self::LangLv),
            "lang_nl" => Ok(Self::LangNl),
            "lang_no" => Ok(Self::LangNo),
            "lang_pl" => Ok(Self::LangPl),
            "lang_pt" => Ok(Self::LangPt),
            "lang_ro" => Ok(Self::LangRo),
            "lang_ru" => Ok(Self::LangRu),
            "lang_sk" => Ok(Self::LangSk),
            "lang_sl" => Ok(Self::LangSl),
            "lang_sr" => Ok(Self::LangSr),
            "lang_sv" => Ok(Self::LangSv),
            "lang_tr" => Ok(Self::LangTr),
            "lang_zh-CN" => Ok(Self::LangZhCN),
            "lang_zh-TW" => Ok(Self::LangZhTW),
            _ => Err("Invalid value"),
        }
    }
}

/// Search Result Number
#[allow(missing_docs)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SearchResultNumber {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
}

impl TryFrom<u32> for SearchResultNumber {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            6 => Ok(Self::Six),
            7 => Ok(Self::Seven),
            8 => Ok(Self::Eight),
            9 => Ok(Self::Nine),
            10 => Ok(Self::Ten),
            _ => Err("Invalid search result number"),
        }
    }
}

impl Display for SearchResultNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::One => write!(f, "1"),
            Self::Two => write!(f, "2"),
            Self::Three => write!(f, "3"),
            Self::Four => write!(f, "4"),
            Self::Five => write!(f, "5"),
            Self::Six => write!(f, "6"),
            Self::Seven => write!(f, "7"),
            Self::Eight => write!(f, "8"),
            Self::Nine => write!(f, "9"),
            Self::Ten => write!(f, "10"),
        }
    }
}

/// Rights Restriction
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Rights {
    /// Creative Commons `Public` Domain
    CcPublicdomain,
    /// Creative Commons `Attribution` (CC BY)
    CcAttribute,
    /// Creative Commons `ShareAlike` (CC SA)
    CcSharealike,
    /// Creative Commons `NoDerivs` (CC ND)
    CcNoncommercial,
    /// Creative Commons `NonCommercial` (CC NC)
    CcNonderived,
    /// And
    And(Box<Rights>, Box<Rights>),
}

impl Display for Rights {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CcPublicdomain => write!(f, "cc_publicdomain"),
            Self::CcAttribute => write!(f, "cc_attribute"),
            Self::CcSharealike => write!(f, "cc_sharealike"),
            Self::CcNoncommercial => write!(f, "cc_noncommercial"),
            Self::CcNonderived => write!(f, "cc_nonderived"),
            Self::And(left, right) => write!(f, "{left}%7C{right}"),
        }
    }
}

/// Safe Search
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Safe {
    /// Active
    Active,
    /// Off
    Off,
}

impl Display for Safe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Off => write!(f, "off"),
        }
    }
}

/// Search Type
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SearchType {
    /// Image
    Image,
}

impl Display for SearchType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Image => write!(f, "image"),
        }
    }
}

/// Site Search Filter
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SiteSearchFilter {
    /// Exclude
    E,
    /// Include
    I,
}

impl Display for SiteSearchFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::E => write!(f, "e"),
            Self::I => write!(f, "i"),
        }
    }
}

/// CSE query parameters
///
/// See <https://developers.google.com/custom-search/v1/reference/rest/v1/cse/list>
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct QueryParameters {
    /// Simplified/Traditional Chinese
    c2coff: Option<C2COff>,
    /// Country restrict(s)
    cr: Option<Box<Cr>>,
    /// Programmable Search Engine ID
    cx: Option<String>,
    /// API key
    key: Option<String>,
    /// Date restrict(s)
    date_restrict: Option<String>,
    /// Exact terms
    exact_terms: Option<String>,
    /// Exclude terms
    exclude_terms: Option<String>,
    /// File type(s)
    file_type: Option<String>,
    /// Filter
    filter: Option<DuplicateContentFilter>,
    /// Geo location of end user
    gl: Option<Gl>,
    /// Low range of search results
    low_range: Option<String>,
    /// High range of search results
    high_range: Option<String>,
    /// Interface language
    hl: Option<String>,
    /// 'And' query terms
    hq: Option<String>,

    /// Image color type
    img_color_type: Option<ImgColorType>,
    /// Image dominant color
    img_dominant_color: Option<ImgDominantColor>,
    /// Image size
    img_size: Option<ImgSize>,
    /// Image type
    img_type: Option<ImgType>,

    /// Link site(s)
    link_site: Option<String>,

    /// Language restrict(s)
    lr: Option<Lr>,

    /// Number of search results to return
    num: Option<SearchResultNumber>,

    /// 'Or' query terms
    or_terms: Option<String>,

    /// Query
    q: Option<String>,

    /// Related site(s)
    related_site: Option<String>,

    /// Rights restrict(s)
    rights: Option<Box<Rights>>,

    /// Search Safety Level
    safe: Option<Safe>,

    /// Search type
    search_type: Option<SearchType>,

    /// Site restrict(s)
    site_search: Option<String>,

    /// Site search filter
    site_search_filter: Option<SiteSearchFilter>,

    /// Sort by
    sort: Option<String>,

    /// Start index
    start: Option<u32>,
}

impl QueryParameters {
    /// Build a `QueryParameters`
    #[must_use]
    pub fn build(&self) -> Self {
        self.clone()
    }

    /// Create a new `QueryParameters`
    #[must_use]
    pub const fn builder() -> Self {
        Self {
            c2coff: None,
            cr: None,
            cx: None,
            key: None,
            date_restrict: None,
            exact_terms: None,
            exclude_terms: None,
            file_type: None,
            filter: None,
            gl: None,
            low_range: None,
            high_range: None,
            hl: None,
            hq: None,
            img_color_type: None,
            img_dominant_color: None,
            img_size: None,
            img_type: None,
            link_site: None,
            lr: None,
            num: None,
            or_terms: None,
            q: None,
            related_site: None,
            rights: None,
            safe: None,
            search_type: None,
            site_search: None,
            site_search_filter: None,
            sort: None,
            start: None,
        }
    }

    /// Convert `QueryParameters` to a Vec of (String, String) tuples
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cognitive_complexity)]
    #[must_use]
    pub fn to_parameters(&self) -> Vec<(String, String)> {
        let mut params = vec![];

        if let Some(key) = &self.key {
            params.push(("key".to_string(), key.to_string()));
        }

        if let Some(cx) = &self.cx {
            params.push(("cx".to_string(), cx.to_string()));
        }

        if let Some(c2coff) = &self.c2coff {
            params.push(("c2coff".to_string(), c2coff.to_string()));
        }

        if let Some(cr) = &self.cr {
            params.push(("cr".to_string(), cr.to_string()));
        }

        if let Some(date_restrict) = &self.date_restrict {
            params.push(("dateRestrict".to_string(), date_restrict.to_string()));
        }

        if let Some(exact_terms) = &self.exact_terms {
            params.push(("exactTerms".to_string(), exact_terms.to_string()));
        }

        if let Some(exclude_terms) = &self.exclude_terms {
            params.push(("excludeTerms".to_string(), exclude_terms.to_string()));
        }

        if let Some(file_type) = &self.file_type {
            params.push(("fileType".to_string(), file_type.to_string()));
        }

        if let Some(filter) = &self.filter {
            params.push(("filter".to_string(), filter.to_string()));
        }

        if let Some(gl) = &self.gl {
            params.push(("gl".to_string(), gl.to_string()));
        }

        if let Some(low_range) = &self.low_range {
            params.push(("lowRange".to_string(), low_range.to_string()));
        }

        if let Some(high_range) = &self.high_range {
            params.push(("highRange".to_string(), high_range.to_string()));
        }

        if let Some(hl) = &self.hl {
            params.push(("hl".to_string(), hl.to_string()));
        }

        if let Some(hq) = &self.hq {
            params.push(("hq".to_string(), hq.to_string()));
        }

        if let Some(img_color_type) = &self.img_color_type {
            params.push(("imgColorType".to_string(), img_color_type.to_string()));
        }

        if let Some(img_dominant_color) = &self.img_dominant_color {
            params.push((
                "imgDominantColor".to_string(),
                img_dominant_color.to_string(),
            ));
        }

        if let Some(img_size) = &self.img_size {
            params.push(("imgSize".to_string(), img_size.to_string()));
        }

        if let Some(img_type) = &self.img_type {
            params.push(("imgType".to_string(), img_type.to_string()));
        }

        if let Some(link_site) = &self.link_site {
            params.push(("linkSite".to_string(), link_site.to_string()));
        }

        if let Some(lr) = &self.lr {
            params.push(("lr".to_string(), lr.to_string()));
        }

        if let Some(num) = &self.num {
            params.push(("num".to_string(), num.to_string()));
        }

        if let Some(or_terms) = &self.or_terms {
            params.push(("orTerms".to_string(), or_terms.to_string()));
        }

        if let Some(q) = &self.q {
            params.push(("q".to_string(), q.to_string()));
        }

        if let Some(related_site) = &self.related_site {
            params.push(("relatedSite".to_string(), related_site.to_string()));
        }

        if let Some(rights) = &self.rights {
            params.push(("rights".to_string(), rights.to_string()));
        }

        if let Some(safe) = &self.safe {
            params.push(("safe".to_string(), safe.to_string()));
        }

        if let Some(search_type) = &self.search_type {
            params.push(("searchType".to_string(), search_type.to_string()));
        }

        if let Some(site_search) = &self.site_search {
            params.push(("siteSearch".to_string(), site_search.to_string()));
        }

        if let Some(site_search_filter) = &self.site_search_filter {
            params.push((
                "siteSearchFilter".to_string(),
                site_search_filter.to_string(),
            ));
        }

        if let Some(sort) = &self.sort {
            params.push(("sort".to_string(), sort.to_string()));
        }

        if let Some(start) = &self.start {
            params.push(("start".to_string(), start.to_string()));
        }

        params
    }

    /// Simplified/Traditional Chinese Search
    pub const fn c2coff(&mut self, c2coff: C2COff) -> &mut Self {
        self.c2coff = Some(c2coff);
        self
    }

    /// Country restrict(s)
    pub fn cr(&mut self, cr: Box<Cr>) -> &mut Self {
        self.cr = Some(cr);
        self
    }

    /// The custom search engine ID to scope this search query
    pub fn cx(&mut self, cx: impl AsRef<str>) -> &mut Self {
        self.cx = Some(cx.as_ref().to_string());
        self
    }

    /// API key
    pub fn key(&mut self, key: impl AsRef<str>) -> &mut Self {
        self.key = Some(key.as_ref().to_string());
        self
    }

    /// Date restrict(s)
    pub fn date_restrict(&mut self, date_restrict: String) -> &mut Self {
        self.date_restrict = Some(date_restrict);
        self
    }

    /// Identifies a phrase that all documents in the search results must
    /// contain
    pub fn exact_terms(&mut self, exact_terms: impl AsRef<str>) -> &mut Self {
        self.exact_terms = Some(exact_terms.as_ref().to_string());
        self
    }

    /// Identifies a word or phrase that should not appear in any documents
    /// in the search results
    pub fn exclude_terms(&mut self, exclude_terms: impl AsRef<str>) -> &mut Self {
        self.exclude_terms = Some(exclude_terms.as_ref().to_string());
        self
    }

    /// Returns images of a specified type. Some of the allowed values are:
    /// bmp, gif, png, jpg, svg, pdf, ...
    pub fn file_type(&mut self, file_type: String) -> &mut Self {
        self.file_type = Some(file_type);
        self
    }

    /// Controls turning on or off the duplicate content filter.
    pub const fn filter(&mut self, filter: DuplicateContentFilter) -> &mut Self {
        self.filter = Some(filter);
        self
    }

    /// Geolocation of end user.
    pub fn gl(&mut self, gl: Gl) -> &mut Self {
        self.gl = Some(gl);
        self
    }

    /// Specifies the ending value for a search range.
    pub fn high_range(&mut self, high_range: String) -> &mut Self {
        self.high_range = Some(high_range);
        self
    }

    /// Sets the user interface language.
    pub fn hl(&mut self, hl: String) -> &mut Self {
        self.hl = Some(hl);
        self
    }

    /// Appends the specified query terms to the query, as if they were
    pub fn hq(&mut self, hq: String) -> &mut Self {
        self.hq = Some(hq);
        self
    }

    /// Restricts results to images of a specified color type.
    pub const fn img_color_type(&mut self, img_color_type: ImgColorType) -> &mut Self {
        self.img_color_type = Some(img_color_type);
        self
    }

    /// Restricts results to images with a specific dominant color.
    pub const fn img_dominant_color(&mut self, img_dominant_color: ImgDominantColor) -> &mut Self {
        self.img_dominant_color = Some(img_dominant_color);
        self
    }

    /// Restricts results to images of a specified size.
    pub const fn img_size(&mut self, img_size: ImgSize) -> &mut Self {
        self.img_size = Some(img_size);
        self
    }

    /// Restricts results to images of a specified type.
    pub const fn img_type(&mut self, img_type: ImgType) -> &mut Self {
        self.img_type = Some(img_type);
        self
    }

    /// Specifies that all search results should contain a link to a
    /// particular URL
    pub fn link_site(&mut self, link_site: String) -> &mut Self {
        self.link_site = Some(link_site);
        self
    }

    /// Restricts the search to documents written in a particular language
    pub const fn lr(&mut self, lr: Lr) -> &mut Self {
        self.lr = Some(lr);
        self
    }

    /// Number of search results to return
    pub const fn num(&mut self, num: SearchResultNumber) -> &mut Self {
        self.num = Some(num);
        self
    }

    /// Provides additional search terms to check for in a document, where
    /// each document in the search results must contain at least one of the
    /// additional search terms
    pub fn or_terms(&mut self, or_terms: String) -> &mut Self {
        self.or_terms = Some(or_terms);
        self
    }

    /// Query
    pub fn q(&mut self, q: impl AsRef<str>) -> &mut Self {
        self.q = Some(q.as_ref().to_string());
        self
    }

    /// Specifies that all search results should be pages that are related
    /// to the specified URL
    pub fn related_site(&mut self, related_site: String) -> &mut Self {
        self.related_site = Some(related_site);
        self
    }

    /// Filters based on licensing. Supported values include:
    /// `cc_publicdomain`, `cc_attribute`, `cc_sharealike`,
    /// `cc_noncommercial`, `cc_nonderived` and combinations of these.
    pub fn rights(&mut self, rights: Rights) -> &mut Self {
        self.rights = Some(Box::new(rights));
        self
    }

    /// Search safety level
    pub const fn safe(&mut self, safe: Safe) -> &mut Self {
        self.safe = Some(safe);
        self
    }

    /// Specifies the search type: image.
    pub const fn search_type(&mut self, search_type: SearchType) -> &mut Self {
        self.search_type = Some(search_type);
        self
    }

    /// Specifies a given site which should always be included or excluded
    /// from results
    pub fn site_search(&mut self, site_search: String) -> &mut Self {
        self.site_search = Some(site_search);
        self
    }

    /// Controls whether to include or exclude results from the site named
    pub const fn site_search_filter(&mut self, site_search_filter: SiteSearchFilter) -> &mut Self {
        self.site_search_filter = Some(site_search_filter);
        self
    }

    /// The sort expression to apply to the results
    pub fn sort(&mut self, sort: String) -> &mut Self {
        self.sort = Some(sort);
        self
    }

    /// The index of the first result to return
    pub const fn start(&mut self, start: u32) -> &mut Self {
        self.start = Some(start);
        self
    }

    /// Build query for next page of results.
    pub const fn next_page(&mut self, n: &NextPage) -> &mut Self {
        self.start = Some(n.start);
        self
    }
}

/// A search result item.
#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    /// The title of the search result, in plain text.
    pub title: String,
    /// The URL of the result.
    pub link: String,
    /// The snippet of the result, in plain text.
    pub snippet: String,
}

/// The next page of results.
pub struct NextPage {
    /// The index of the first result.
    pub start: u32,
}

/// A search result.
///
/// The complete documentation is available at <https://developers.google.com/custom-search/v1/reference/rest/v1/Search>
#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResults {
    /// The search results.
    pub items: Vec<Item>,

    #[serde(flatten)]
    other: Map<String, Value>,
}

impl SearchResults {
    /// Returns the [`NextPage`] so one can construct a new query to get the
    /// next page of results.
    ///
    /// See [`QueryParameters::next_page`].
    #[must_use]
    pub fn next_page(&self) -> Option<NextPage> {
        self.other
            .get("queries")
            .and_then(|queries| queries.get("nextPage"))
            .and_then(|next_page| next_page.get(0))
            .and_then(|next_page| next_page.get("startIndex"))
            .and_then(serde_json::Value::as_u64)
            .map(|start_index| NextPage {
                start: start_index as u32,
            })
    }
}
