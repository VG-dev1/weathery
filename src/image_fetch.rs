use anyhow::Result;
use image::DynamicImage;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct WikiResponse {
    query: WikiQuery,
}

#[derive(Deserialize)]
struct WikiQuery {
    pages: HashMap<String, WikiPage>,
}

#[derive(Deserialize)]
struct WikiSearchResponse {
    query: WikiSearchQuery,
}

#[derive(Deserialize)]
struct WikiSearchQuery {
    search: Vec<WikiSearchItem>,
}

#[derive(Deserialize)]
struct WikiSearchItem {
    title: String,
}

#[derive(Deserialize)]
struct WikiPage {
    thumbnail: Option<WikiThumbnail>,
}

#[derive(Deserialize)]
struct WikiThumbnail {
    source: String,
}

pub async fn get_city_image_url(city: &str) -> Result<Option<String>> {
    let client = reqwest::Client::builder()
        .user_agent("weathery/1.0 (contact@example.com)")
        .build()?;

    if let Some(url) = get_thumbnail_for_title(&client, city).await? {
        return Ok(Some(url));
    }

    let search: WikiSearchResponse = client
        .get("https://en.wikipedia.org/w/api.php")
        .query(&[
            ("action", "query"),
            ("format", "json"),
            ("list", "search"),
            ("srsearch", city),
            ("srlimit", "8"),
        ])
        .send()
        .await?
        .json()
        .await?;

    for item in search.query.search {
        if let Some(url) = get_thumbnail_for_title(&client, &item.title).await? {
            return Ok(Some(url));
        }
    }

    Ok(None)
}

async fn get_thumbnail_for_title(client: &reqwest::Client, title: &str) -> Result<Option<String>> {
    let response: WikiResponse = client
        .get("https://en.wikipedia.org/w/api.php")
        .query(&[
            ("action", "query"),
            ("format", "json"),
            ("titles", title),
            ("prop", "pageimages"),
            ("piprop", "thumbnail"),
            ("pithumbsize", "1000"),
            ("redirects", "1"),
        ])
        .send()
        .await?
        .json()
        .await?;

    let url = response
        .query
        .pages
        .values()
        .find_map(|page| page.thumbnail.as_ref().map(|t| t.source.clone()));

    Ok(url)
}

pub async fn download_image(url: &str) -> Result<DynamicImage> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .build()?;

    let bytes = client.get(url).send().await?.bytes().await?;
    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}
