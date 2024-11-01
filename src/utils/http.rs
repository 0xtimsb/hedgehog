use reqwest::{Client, Url};

pub async fn get_downloadable_content_type(url: &str) -> Option<String> {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .ok()?;

    let response = client.get(Url::parse(url).ok()?).send().await.ok()?;

    if let Some(content_type) = response
        .headers()
        .get("content-type")
        .or_else(|| response.headers().get("Content-Type"))
        .and_then(|ct| ct.to_str().ok())
    {
        if !content_type.starts_with("text/")
            && ![
                "application/javascript",
                "application/json",
                "application/xml",
            ]
            .contains(&content_type)
        {
            return Some(content_type.to_string());
        }
    }

    response
        .headers()
        .get("content-disposition")
        .and_then(|d| d.to_str().ok())
        .filter(|d| d.contains("attachment"))
        .map(|_| "attachment".to_string())
}
