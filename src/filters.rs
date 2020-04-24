use warp::Filter;

pub fn kcproxy() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    kcproxy_login()
        .or(entry())
        .or(kcsapi())
        .or(cache_or_proxy("gadget_html5"))
        .or(cache_or_proxy("html"))
        .or(cache_or_proxy("kcscontents"))
        .or(cache_or_proxy("kcs2"))
        .or(cache_or_proxy("kcs"))
        .or(spa())
        .with(warp::compression::gzip())
}

pub fn entry() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("entry")
        .and(warp::get())
        .and(warp::cookie("token"))
        .and_then(super::handlers::entry)
}

pub fn kcsapi() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("kcsapi")
        .and(warp::post())
        .and(warp::cookie("token"))
        .and(warp::header("referer"))
        .and(warp::path::full())
        .and(warp::body::form())
        .and_then(super::handlers::kcsapi)
}

pub fn cache_or_proxy(
    dir: &'static str,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let path = format!("./cache/{}", dir);
    let cache = warp::fs::dir(path);
    let proxy = warp::cookie("token")
        .and(warp::header::optional("referer"))
        .and(warp::path::full())
        .and(warp::query::raw())
        .and_then(super::handlers::kcs_static);

    warp::path(dir)
        .and(warp::any())
        .and(warp::get())
        .and(cache.or(proxy))
}

pub fn kcproxy_login() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("login")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and_then(super::handlers::login)
}

pub fn spa() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let kancolle = warp::path("kancolle")
        .and(warp::get())
        .and(warp::fs::file("./static/index.html"));
    let index = warp::path::end()
        .and(warp::get())
        .and(warp::fs::file("./static/index.html"));
    let any = warp::any().and(warp::get()).and(warp::fs::dir("./static"));
    kancolle.or(index).or(any)
}
