use chrono::{prelude::*, Duration};
use hyper::{Client, Request};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::Infallible};
use warp::path::FullPath;

#[derive(Deserialize, Serialize)]
pub struct UserToken {
    pub world_id: usize,
    pub api_token: String,
    pub api_starttime: i64,
    pub exp: i64,
}

pub fn get_secret_key() -> &'static str {
    lazy_static! {
        static ref SECRET_KEY: String = std::env::var_os("SECRET_KEY")
            .unwrap()
            .into_string()
            .unwrap();
    }

    &SECRET_KEY
}

pub async fn login(user: super::auth::DmmUser) -> Result<Box<dyn warp::Reply>, Infallible> {
    let kancolle_token: super::auth::KancolleToken = match super::auth::get_token(&user).await {
        Ok(token) => token,
        Err(_err) => return Ok(Box::new("Try again")),
    };
    let world_id = kancolle_token.world_id;
    let api_token = kancolle_token.api_token;
    let api_starttime = kancolle_token.api_starttime;
    let claims = UserToken {
        world_id,
        api_token,
        api_starttime,
        exp: (Local::now() + Duration::days(1)).timestamp(),
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(get_secret_key().as_ref()),
    )
    .unwrap();
    let cookie = format!("token={}; httpOnly", token);
    Ok(Box::new(warp::reply::with_header(
        warp::reply(),
        "Set-Cookie",
        cookie,
    )))
}

pub fn decode_token(token: String) -> UserToken {
    let claims = jsonwebtoken::decode::<UserToken>(
        &token,
        &jsonwebtoken::DecodingKey::from_secret(get_secret_key().as_ref()),
        &jsonwebtoken::Validation::default(),
    )
    .unwrap()
    .claims;

    claims
}

pub async fn entry(token: UserToken) -> Result<impl warp::Reply, Infallible> {
    let reply = format!(
        "https://kc.icicle.moe/kcs2/index.php?api_root=/kcsapi&voice_root=/kcs/sound&osapi_root=osapi.dmm.com&version=4.5.6.2&api_token={api_token}&api_starttime={api_starttime}",
        api_token = token.api_token,
        api_starttime = token.api_starttime,
    );

    Ok(reply)
}

pub async fn kcsapi(
    token: UserToken,
    referer: String,
    path: FullPath,
    body: HashMap<String, String>,
) -> Result<impl warp::Reply, Infallible> {
    let world_id = token.world_id;
    let world_ip = super::auth::get_world_ip(world_id);
    let url = format!("http://{}", &world_ip);
    let mut url = Url::parse(&url).unwrap();
    url.set_path(path.as_str());

    let client = reqwest::Client::builder()
        .user_agent(super::auth::USER_AGENT)
        .build()
        .unwrap();

    let text = client
        .post(url)
        .header("Referer", replace_referer_host(&referer, &world_ip))
        .form(&body)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    Ok(text)
}

pub async fn kcs_static(
    token: UserToken,
    referer: Option<String>,
    path: FullPath,
    query: String,
) -> Result<impl warp::Reply, Infallible> {
    let world_id = token.world_id;
    let world_ip = super::auth::get_world_ip(world_id);
    let url = format!("http://{}", &world_ip);
    let mut url = Url::parse(&url).unwrap();
    url.set_path(path.as_str());
    url.set_query(Some(&query));

    let client = Client::new();
    let req = Request::get(url.as_str()).header("User-Agent", super::auth::USER_AGENT);
    let req = match referer {
        Some(referer) => req.header(
            "Referer",
            replace_referer_host(&referer, &world_ip).as_str(),
        ),
        None => req,
    };
    let req = req.body(hyper::Body::empty()).unwrap();

    let res = client.request(req).await.unwrap();

    Ok(res)
}

pub fn replace_referer_host(referer: &str, host: &str) -> String {
    let mut referer_url = Url::parse(referer).unwrap();
    referer_url.set_host(Some(host)).unwrap();
    referer_url.set_port(Some(80)).unwrap();

    referer_url.as_str().to_string()
}
