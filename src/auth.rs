use regex::Regex;
use reqwest::header::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.92 Safari/537.36";
static DMM_LOGIN_URL: &str = "https://accounts.dmm.com/service/login/password/=/";
static DMM_API_URL: &str = "https://accounts.dmm.com/service/api/get-token/";
static DMM_AUTH_URL: &str = "https://accounts.dmm.com/service/login/password/authenticate/";
static DMM_GAME_URL: &str = "http://www.dmm.com/netgame/social/-/gadgets/=/app_id=854854/";
static DMM_MAKE_REQUEST: &str = "http://osapi.dmm.com/gadgets/makeRequest/";

#[derive(Debug, Deserialize, Serialize)]
pub struct DmmUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DmmToken {
    dmm_token: String,
    token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DmmApiToken {
    token: String,
    // id_key: String,
    // pw_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DmmOsapiQuery {
    owner: String,
    st: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KancolleToken {
    pub world_id: usize,
    pub api_token: String,
    pub api_starttime: i64,
}

pub fn get_world_ip(world_id: usize) -> String {
    lazy_static! {
        static ref KANCOLLE_WORLDS: Vec<&'static str> = vec![
            "203.104.209.71",
            "203.104.209.87",
            "125.6.184.215",
            "203.104.209.183",
            "203.104.209.150",
            "203.104.209.134",
            "203.104.209.167",
            "203.104.209.199",
            "125.6.189.7",
            "125.6.189.39",
            "125.6.189.71",
            "125.6.189.103",
            "125.6.189.135",
            "125.6.189.167",
            "125.6.189.215",
            "125.6.189.247",
            "203.104.209.23",
            "203.104.209.39",
            "203.104.209.55",
            "203.104.209.102"
        ];
    }

    return KANCOLLE_WORLDS[world_id - 1].to_string();
}

pub async fn get_dmm_tokens() -> Result<DmmToken, Box<dyn std::error::Error>> {
    lazy_static! {
        static ref DMM_TOKEN_RE: Regex =
            Regex::new(r#"csrf-http-dmm-token" content="([\d|\w]+)""#).unwrap();
        static ref TOKEN_RE: Regex = Regex::new(r#"csrf-token" content="([\d|\w]+)""#).unwrap();
    }

    let html = reqwest::get(DMM_LOGIN_URL).await?.text().await?;
    let dmm_token = DMM_TOKEN_RE
        .captures(&html)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str();
    let token = TOKEN_RE.captures(&html).unwrap().get(1).unwrap().as_str();

    let dmm_token = DmmToken {
        dmm_token: dmm_token.to_string(),
        token: token.to_string(),
    };

    Ok(dmm_token)
}

pub async fn get_api_token(
    client: &reqwest::Client,
    dmm_token: &DmmToken,
) -> Result<DmmApiToken, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(ORIGIN, "https://accounts.dmm.com".parse()?);
    headers.insert(REFERER, DMM_LOGIN_URL.parse()?);
    headers.insert(
        HeaderName::from_lowercase(b"http-dmm-token")?,
        dmm_token.dmm_token.parse()?,
    );
    headers.insert(
        HeaderName::from_lowercase(b"x-requested-with")?,
        "XMLHttpRequest".parse()?,
    );

    let mut map = HashMap::new();
    map.insert("token", &dmm_token.token);

    let res: HashMap<String, serde_json::Value> = client
        .post(DMM_API_URL)
        .headers(headers)
        .json(&map)
        .send()
        .await?
        .json()
        .await?;
    let token = res["body"]["token"].as_str().unwrap().to_string();
    // let id_key = res["body"]["login_id"].to_string();
    // let pw_key = res["body"]["password"].to_string();

    let dmm_api_token = DmmApiToken {
        token,
        // id_key,
        // pw_key,
    };

    Ok(dmm_api_token)
}

pub async fn get_osapi_url(
    client: &reqwest::Client,
    api_token: &DmmApiToken,
    username: &str,
    password: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    lazy_static! {
        static ref OSAPI_URL_RE: Regex = Regex::new(r#"URL\W+:\W+"(.*)","#).unwrap();
    }

    let mut headers = HeaderMap::new();
    headers.insert(ORIGIN, "https://accounts.dmm.com".parse()?);
    headers.insert(REFERER, DMM_LOGIN_URL.parse()?);

    let mut map = HashMap::new();
    map.insert("token", api_token.token.as_str());
    map.insert("login_id", username);
    map.insert("password", password);
    map.insert("idKey", username);
    map.insert("pwKey", password);
    map.insert("path", "");
    map.insert("prompt", "");

    // Login to DMM
    client
        .post(DMM_AUTH_URL)
        .headers(headers.clone())
        .json(&map)
        .send()
        .await?;

    let html = client
        .get(DMM_GAME_URL)
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    let url = OSAPI_URL_RE.captures(&html).unwrap().get(1).unwrap();

    Ok(url.as_str().to_string())
}

pub async fn parse_osapi_url(url: &str) -> Result<DmmOsapiQuery, Box<dyn std::error::Error>> {
    let url = reqwest::Url::parse(url)?;
    let query = url.query_pairs().collect::<HashMap<_, _>>();

    Ok(DmmOsapiQuery {
        st: query.get("st").unwrap().to_string(),
        owner: query.get("owner").unwrap().to_string(),
    })
}

pub async fn get_world_id(
    client: &reqwest::Client,
    osapi_url: &str,
    osapi_query: &DmmOsapiQuery,
) -> Result<usize, Box<dyn std::error::Error>> {
    let request_url = format!(
        "http://203.104.209.7/kcsapi/api_world/get_id/{owner}/1/{timestamp}",
        owner = osapi_query.owner,
        timestamp = chrono::Local::now().timestamp_millis(),
    );

    let mut headers = HeaderMap::new();
    headers.insert(ORIGIN, "https://osapi.dmm.com".parse()?);
    headers.insert(REFERER, osapi_url.parse()?);

    let query = &[
        ("refresh", "3600"),
        ("url", request_url.as_str()),
        ("httpMethod", "GET"),
        ("headers", ""),
        ("postData", ""),
        ("authz", ""),
        ("st", ""),
        ("contentType", "JSON"),
        ("numEntries", "3"),
        ("getSummaries", "false"),
        ("signOwner", "true"),
        ("signViewer", "true"),
        ("gadget", "http://203.104.209.7/gadget_html5.xml"),
        ("container", "dmm"),
        ("bypassSpecCache", ""),
        ("getFullHeaders", "false"),
    ];

    let text = client
        .get(DMM_MAKE_REQUEST)
        .headers(headers)
        .query(query)
        .send()
        .await?
        .text()
        .await?;

    let svdata = parse_svdata(&text)?;
    let world_id: i64 = svdata["api_data"]["api_world_id"].as_i64().unwrap();

    Ok(world_id as usize)
}

pub async fn get_kancolle_token(
    client: &reqwest::Client,
    osapi_url: &str,
    osapi_query: &DmmOsapiQuery,
    world_id: usize,
) -> Result<KancolleToken, Box<dyn std::error::Error>> {
    lazy_static! {
        static ref TOKEN_RE: Regex = Regex::new(r#"([\d|\w]+)"#).unwrap();
    }

    let world_ip = get_world_ip(world_id);
    let request_url = format!(
        "http://{world_ip}/kcsapi/api_auth_member/dmmlogin/{owner}/1/{timestamp}",
        world_ip = world_ip,
        owner = osapi_query.owner,
        timestamp = chrono::Local::now().timestamp_millis(),
    );

    let mut headers = HeaderMap::new();
    headers.insert(ORIGIN, "https://osapi.dmm.com".parse()?);
    headers.insert(REFERER, osapi_url.parse()?);

    let data = &[
        ("url", request_url.as_str()),
        ("httpMethod", "GET"),
        ("headers", ""),
        ("postData", ""),
        ("authz", "signed"),
        ("st", &osapi_query.st),
        ("contentType", "JSON"),
        ("numEntries", "3"),
        ("getSummaries", "false"),
        ("signOwner", "true"),
        ("signViewer", "true"),
        ("gadget", "http://203.104.209.7/gadget_html5.xml"),
        ("container", "dmm"),
        ("bypassSpecCache", ""),
        ("getFullHeaders", "false"),
        ("oauthState", ""),
    ];

    let text: String = client
        .post(DMM_MAKE_REQUEST)
        .headers(headers)
        .form(data)
        .send()
        .await?
        .text()
        .await?;

    let svdata = parse_svdata(&text)?;
    let api_starttime = svdata["api_starttime"].as_i64().unwrap();
    let api_token = svdata["api_token"].as_str().unwrap();
    let api_token = TOKEN_RE
        .captures(api_token)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .to_string();

    Ok(KancolleToken {
        world_id,
        api_starttime,
        api_token,
    })
}

pub async fn get_token(user: &DmmUser) -> Result<KancolleToken, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .cookie_store(true)
        .build()
        .unwrap();

    let dmm_token = get_dmm_tokens().await?;
    let dmm_api_token = get_api_token(&client, &dmm_token).await?;
    let osapi_url = get_osapi_url(&client, &dmm_api_token, &user.username, &user.password).await?;
    let osapi_query = parse_osapi_url(&osapi_url).await?;
    let world_id = get_world_id(&client, &osapi_url, &osapi_query).await?;
    let kancolle_token = get_kancolle_token(&client, &osapi_url, &osapi_query, world_id).await?;

    Ok(kancolle_token)
}

pub fn parse_svdata(
    text: &str,
) -> Result<serde_json::Map<String, serde_json::Value>, Box<dyn std::error::Error>> {
    let slice = &text[27..];

    let v: serde_json::Map<String, serde_json::Value> = serde_json::from_str(slice)?;
    let o = v.values().next().unwrap().as_object().unwrap();
    let body = &o["body"].as_str().unwrap()[7..];
    let svdata: serde_json::Map<String, serde_json::Value> = serde_json::from_str(body)?;

    Ok(svdata)
}

#[tokio::test]
async fn test_get_dmm_token() {
    let token = get_dmm_tokens().await.unwrap();
    assert_eq!(token.dmm_token.len(), 32);
    assert_eq!(token.token.len(), 32);
}

#[tokio::test]
async fn check_my_ip() {
    let res = reqwest::get("https://api.myip.com")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("{}", res);
}
