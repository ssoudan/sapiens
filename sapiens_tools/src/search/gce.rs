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
            C2COff::Off => write!(f, "0"),
            C2COff::On => write!(f, "1"),
        }
    }
}

/// Country Restriction
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Cr {
    /// Country code - See https://developers.google.com/custom-search/docs/json_api_reference#countryCollections
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
            Cr::Country(country) => write!(f, "{}", country),
            Cr::Not(cr) => write!(f, "-{}", cr),
            Cr::And(cr1, cr2) => write!(f, "({}).({})", cr1, cr2),
            Cr::Or(cr1, cr2) => write!(f, "({}|{})", cr1, cr2),
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
            DateRestrict::Day(day) => write!(f, "d{}", day),
            DateRestrict::Week(week) => write!(f, "w{}", week),
            DateRestrict::Month(month) => write!(f, "m{}", month),
            DateRestrict::Year(year) => write!(f, "y{}", year),
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
            DuplicateContentFilter::Off => write!(f, "0"),
            DuplicateContentFilter::On => write!(f, "1"),
        }
    }
}

/// Geolocation of end user
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Gl {
    /// Country code - See https://developers.google.com/custom-search/docs/json_api_reference#country-codes
    /// (2 letters lower case).
    CountryCode(String),
}

impl Display for Gl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Gl::CountryCode(country_code) => write!(f, "{}", country_code),
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
            ImgColorType::Color => {
                write!(f, "color")
            }
            ImgColorType::Gray => {
                write!(f, "gray")
            }
            ImgColorType::Mono => {
                write!(f, "mono")
            }
            ImgColorType::Trans => {
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
            ImgDominantColor::Black => {
                write!(f, "black")
            }
            ImgDominantColor::Blue => {
                write!(f, "blue")
            }
            ImgDominantColor::Brown => {
                write!(f, "brown")
            }
            ImgDominantColor::Gray => {
                write!(f, "gray")
            }
            ImgDominantColor::Green => {
                write!(f, "green")
            }
            ImgDominantColor::Orange => {
                write!(f, "orange")
            }
            ImgDominantColor::Pink => {
                write!(f, "pink")
            }
            ImgDominantColor::Purple => {
                write!(f, "purple")
            }
            ImgDominantColor::Red => {
                write!(f, "red")
            }
            ImgDominantColor::Teal => {
                write!(f, "teal")
            }
            ImgDominantColor::White => {
                write!(f, "white")
            }
            ImgDominantColor::Yellow => {
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
            ImgSize::Icon => {
                write!(f, "icon")
            }
            ImgSize::Small => {
                write!(f, "small")
            }
            ImgSize::Medium => {
                write!(f, "medium")
            }
            ImgSize::Large => {
                write!(f, "large")
            }
            ImgSize::Xlarge => {
                write!(f, "xlarge")
            }
            ImgSize::Xxlarge => {
                write!(f, "xxlarge")
            }
            ImgSize::Huge => {
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
            ImgType::Face => {
                write!(f, "face")
            }
            ImgType::Photo => {
                write!(f, "photo")
            }
            ImgType::Clipart => {
                write!(f, "clipart")
            }
            ImgType::Lineart => {
                write!(f, "lineart")
            }
            ImgType::Animated => {
                write!(f, "animated")
            }
            ImgType::Stock => {
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
            Lr::LangAr => write!(f, "lang_ar"),
            Lr::LangBg => write!(f, "lang_bg"),
            Lr::LangCa => write!(f, "lang_ca"),
            Lr::LangCs => write!(f, "lang_cs"),
            Lr::LangDa => write!(f, "lang_da"),
            Lr::LangDe => write!(f, "lang_de"),
            Lr::LangEl => write!(f, "lang_el"),
            Lr::LangEn => write!(f, "lang_en"),
            Lr::LangEs => write!(f, "lang_es"),
            Lr::LangEt => write!(f, "lang_et"),
            Lr::LangFi => write!(f, "lang_fi"),
            Lr::LangFr => write!(f, "lang_fr"),
            Lr::LangHr => write!(f, "lang_hr"),
            Lr::LangHu => write!(f, "lang_hu"),
            Lr::LangId => write!(f, "lang_id"),
            Lr::LangIs => write!(f, "lang_is"),
            Lr::LangIt => write!(f, "lang_it"),
            Lr::LangIw => write!(f, "lang_iw"),
            Lr::LangJa => write!(f, "lang_ja"),
            Lr::LangKo => write!(f, "lang_ko"),
            Lr::LangLt => write!(f, "lang_lt"),
            Lr::LangLv => write!(f, "lang_lv"),
            Lr::LangNl => write!(f, "lang_nl"),
            Lr::LangNo => write!(f, "lang_no"),
            Lr::LangPl => write!(f, "lang_pl"),
            Lr::LangPt => write!(f, "lang_pt"),
            Lr::LangRo => write!(f, "lang_ro"),
            Lr::LangRu => write!(f, "lang_ru"),
            Lr::LangSk => write!(f, "lang_sk"),
            Lr::LangSl => write!(f, "lang_sl"),
            Lr::LangSr => write!(f, "lang_sr"),
            Lr::LangSv => write!(f, "lang_sv"),
            Lr::LangTr => write!(f, "lang_tr"),
            Lr::LangZhCN => write!(f, "lang_zh-CN"),
            Lr::LangZhTW => write!(f, "lang_zh-TW"),
        }
    }
}

impl TryFrom<&str> for Lr {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "lang_ar" => Ok(Lr::LangAr),
            "lang_bg" => Ok(Lr::LangBg),
            "lang_ca" => Ok(Lr::LangCa),
            "lang_cs" => Ok(Lr::LangCs),
            "lang_da" => Ok(Lr::LangDa),
            "lang_de" => Ok(Lr::LangDe),
            "lang_el" => Ok(Lr::LangEl),
            "lang_en" => Ok(Lr::LangEn),
            "lang_es" => Ok(Lr::LangEs),
            "lang_et" => Ok(Lr::LangEt),
            "lang_fi" => Ok(Lr::LangFi),
            "lang_fr" => Ok(Lr::LangFr),
            "lang_hr" => Ok(Lr::LangHr),
            "lang_hu" => Ok(Lr::LangHu),
            "lang_id" => Ok(Lr::LangId),
            "lang_is" => Ok(Lr::LangIs),
            "lang_it" => Ok(Lr::LangIt),
            "lang_iw" => Ok(Lr::LangIw),
            "lang_ja" => Ok(Lr::LangJa),
            "lang_ko" => Ok(Lr::LangKo),
            "lang_lt" => Ok(Lr::LangLt),
            "lang_lv" => Ok(Lr::LangLv),
            "lang_nl" => Ok(Lr::LangNl),
            "lang_no" => Ok(Lr::LangNo),
            "lang_pl" => Ok(Lr::LangPl),
            "lang_pt" => Ok(Lr::LangPt),
            "lang_ro" => Ok(Lr::LangRo),
            "lang_ru" => Ok(Lr::LangRu),
            "lang_sk" => Ok(Lr::LangSk),
            "lang_sl" => Ok(Lr::LangSl),
            "lang_sr" => Ok(Lr::LangSr),
            "lang_sv" => Ok(Lr::LangSv),
            "lang_tr" => Ok(Lr::LangTr),
            "lang_zh-CN" => Ok(Lr::LangZhCN),
            "lang_zh-TW" => Ok(Lr::LangZhTW),
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
            1 => Ok(SearchResultNumber::One),
            2 => Ok(SearchResultNumber::Two),
            3 => Ok(SearchResultNumber::Three),
            4 => Ok(SearchResultNumber::Four),
            5 => Ok(SearchResultNumber::Five),
            6 => Ok(SearchResultNumber::Six),
            7 => Ok(SearchResultNumber::Seven),
            8 => Ok(SearchResultNumber::Eight),
            9 => Ok(SearchResultNumber::Nine),
            10 => Ok(SearchResultNumber::Ten),
            _ => Err("Invalid search result number"),
        }
    }
}

impl Display for SearchResultNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchResultNumber::One => write!(f, "1"),
            SearchResultNumber::Two => write!(f, "2"),
            SearchResultNumber::Three => write!(f, "3"),
            SearchResultNumber::Four => write!(f, "4"),
            SearchResultNumber::Five => write!(f, "5"),
            SearchResultNumber::Six => write!(f, "6"),
            SearchResultNumber::Seven => write!(f, "7"),
            SearchResultNumber::Eight => write!(f, "8"),
            SearchResultNumber::Nine => write!(f, "9"),
            SearchResultNumber::Ten => write!(f, "10"),
        }
    }
}

/// Rights Restriction
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Rights {
    /// Creative Commons Public Domain
    CcPublicdomain,
    /// Creative Commons Attribution (CC BY)
    CcAttribute,
    /// Creative Commons ShareAlike (CC SA)
    CcSharealike,
    /// Creative Commons NoDerivs (CC ND)
    CcNoncommercial,
    /// Creative Commons NonCommercial (CC NC)
    CcNonderived,
    /// And
    And(Box<Rights>, Box<Rights>),
}

impl Display for Rights {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Rights::CcPublicdomain => write!(f, "cc_publicdomain"),
            Rights::CcAttribute => write!(f, "cc_attribute"),
            Rights::CcSharealike => write!(f, "cc_sharealike"),
            Rights::CcNoncommercial => write!(f, "cc_noncommercial"),
            Rights::CcNonderived => write!(f, "cc_nonderived"),
            Rights::And(left, right) => write!(f, "{}%7C{}", left, right),
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
            Safe::Active => write!(f, "active"),
            Safe::Off => write!(f, "off"),
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
            SearchType::Image => write!(f, "image"),
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
            SiteSearchFilter::E => write!(f, "e"),
            SiteSearchFilter::I => write!(f, "i"),
        }
    }
}

/// CSE query parameters
///
/// See https://developers.google.com/custom-search/v1/reference/rest/v1/cse/list
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
    /// Build a QueryParameters
    pub fn build(&mut self) -> Self {
        self.clone()
    }

    /// Create a new QueryParameters
    pub fn builder() -> QueryParameters {
        QueryParameters {
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

    /// Convert QueryParameters to a Vec of (String, String) tuples
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
    pub fn c2coff(&mut self, c2coff: C2COff) -> &mut Self {
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
    pub fn filter(&mut self, filter: DuplicateContentFilter) -> &mut Self {
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
    pub fn img_color_type(&mut self, img_color_type: ImgColorType) -> &mut Self {
        self.img_color_type = Some(img_color_type);
        self
    }

    /// Restricts results to images with a specific dominant color.
    pub fn img_dominant_color(&mut self, img_dominant_color: ImgDominantColor) -> &mut Self {
        self.img_dominant_color = Some(img_dominant_color);
        self
    }

    /// Restricts results to images of a specified size.
    pub fn img_size(&mut self, img_size: ImgSize) -> &mut Self {
        self.img_size = Some(img_size);
        self
    }

    /// Restricts results to images of a specified type.
    pub fn img_type(&mut self, img_type: ImgType) -> &mut Self {
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
    pub fn lr(&mut self, lr: Lr) -> &mut Self {
        self.lr = Some(lr);
        self
    }

    /// Number of search results to return
    pub fn num(&mut self, num: SearchResultNumber) -> &mut Self {
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
    /// cc_publicdomain, cc_attribute, cc_sharealike,
    /// cc_noncommercial, cc_nonderived and combinations of these.
    pub fn rights(&mut self, rights: Rights) -> &mut Self {
        self.rights = Some(Box::new(rights));
        self
    }

    /// Search safety level
    pub fn safe(&mut self, safe: Safe) -> &mut Self {
        self.safe = Some(safe);
        self
    }

    /// Specifies the search type: image.
    pub fn search_type(&mut self, search_type: SearchType) -> &mut Self {
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
    pub fn site_search_filter(&mut self, site_search_filter: SiteSearchFilter) -> &mut Self {
        self.site_search_filter = Some(site_search_filter);
        self
    }

    /// The sort expression to apply to the results
    pub fn sort(&mut self, sort: String) -> &mut Self {
        self.sort = Some(sort);
        self
    }

    /// The index of the first result to return
    pub fn start(&mut self, start: u32) -> &mut Self {
        self.start = Some(start);
        self
    }

    /// Build query for next page of results.
    pub fn next_page(&mut self, n: NextPage) -> &mut Self {
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
/// The complete documentation is available at https://developers.google.com/custom-search/v1/reference/rest/v1/Search
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
    pub fn next_page(&self) -> Option<NextPage> {
        self.other
            .get("queries")
            .and_then(|queries| queries.get("nextPage"))
            .and_then(|next_page| next_page.get(0))
            .and_then(|next_page| next_page.get("startIndex"))
            .and_then(|start_index| start_index.as_u64())
            .map(|start_index| NextPage {
                start: start_index as u32,
            })
    }
}
