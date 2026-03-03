#[derive(Debug, Clone, Eq, PartialEq)]
struct Rsquest {
        id: String,
        method: String,
        headers: Vec<(String, String)>,
        body: String,
}

struct RequestBuilder {
        id: Option<String>,
        method: Option<String>,
        headers: Option<Vec<(String, String)>>,
        body: Option<String>,
}
