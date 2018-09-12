extern crate serde;
extern crate serde_json;

use actix_web::middleware::Finished;
use actix_web::middleware::Middleware;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use options::ProxyOpts;
use url::Url;

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Data {
    url: String,
    id: String,
    referrer: String,
}

fn extract_data(url: &str) -> Option<Data> {
    let url = Url::parse(url).ok()?;

    let matched = url
        .query_pairs()
        .find(|(key, _)| key == "bs_track")
        .map(|(_, value)| value)?;

    let d: Result<Data, _> = serde_json::from_str(&matched);

    match d {
        Ok(data) => Some(data),
        Err(e) => {
            println!("{:?}", e);
            None
        }
    }
}

pub struct ReqCatcher {
    name: String,
}

impl ReqCatcher {
    pub fn new() -> ReqCatcher {
        ReqCatcher {
            name: "bs".to_string(),
        }
    }
}

impl<S> Middleware<S> for ReqCatcher {
    fn finish(&self, req: &HttpRequest<S>, resp: &HttpResponse) -> Finished {
        println!("{:?}", extract_data(&req.uri().to_string()));
        Finished::Done
    }
}

pub struct M2Prest {
    pub resources: Vec<(String, fn(&HttpRequest<ProxyOpts>) -> HttpResponse)>,
    pub middleware: Vec<Box<Middleware<ProxyOpts>>>
}

impl M2Prest {
    pub fn new() -> M2Prest {
        M2Prest {
            resources: vec![(
                String::from(
                    "/static/{version}/frontend/{vendor}/{theme}/{locale}/requirejs/require.js",
                ),
                serve_instrumented_require_js,
            )],
            middleware: vec![
                Box::new(ReqCatcher::new())
            ]
        }
    }
}

/// handler with path parameters like `/user/{name}/`
fn serve_instrumented_require_js(req: &HttpRequest<ProxyOpts>) -> HttpResponse {
    println!("{:?}", req);
    let bytes = include_str!("./static/requirejs.js");

    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extract_data() {
        let url = "https://127.0.0.1:8080/static/version1536567404/frontend/Acme/default/en_GB/Magento_Ui/js/form/form.js?bs_track=%7B%22url%22%3A%22https%3A%2F%2F127.0.0.1%3A8080%2Fstatic%2Fversion1536567404%2Ffrontend%2FAcme%2Fdefault%2Fen_GB%2FMagento_Ui%2Fjs%2Fform%2Fform.js%22%2C%22id%22%3A%22Magento_Ui%2Fjs%2Fform%2Fform%22%2C%22referrer%22%3A%22%2F%22%7D";
        let d = extract_data(url).unwrap();
        assert_eq!(d, Data{
            url: String::from("https://127.0.0.1:8080/static/version1536567404/frontend/Acme/default/en_GB/Magento_Ui/js/form/form.js"),
            id: String::from("Magento_Ui/js/form/form"),
            referrer: String::from("/")
        })
    }
}