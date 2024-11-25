use std::{
    io::{Cursor, Write},
    str::FromStr,
    time::Duration,
};

const XSR_TOKEN : str = todo!();
const OSU_SESSION : str = todo!();

#[tokio::main]
async fn main() {
    std::fs::write("log.txt", "").unwrap();
    let mut log_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("log.txt")
        .expect("could not init log file!");

        
    let mut start = 419697;
    let max = 3000000;
    let start_wait_time = 61;
    let mut wait_time = start_wait_time;
    while start <= max {
        let url = format!("https://osu.ppy.sh/beatmapsets/{}/download", start);
        println!("{:#?}: {start}", chrono::Utc::now());
        let status_res = download_file(url.clone()).await;
        println!("going to url{}", &url);
        if let Ok(status) = status_res {
            if status == 429 {
                //waiting if sent too many requests.
                println!(
                    "{:#?}: waiting {wait_time} seconds to get more requests",
                    chrono::Utc::now()
                );
                log_file
                    .write_all(format!("{start}, {:#?}\n", chrono::Utc::now()).as_bytes())
                    .expect("could not write to log file!");

                tokio::time::sleep(Duration::from_secs(wait_time)).await;
                wait_time *= 2;
                continue;
            }

            wait_time = start_wait_time;
            log_file
                .write_all(format!("{start}, {status}\n").as_bytes())
                .expect("could not write to log file!");
        } else {
            log_file
                .write_all(format!("{start}, EXITED WITH ERROR!\n").as_bytes())
                .expect("could not write to log file!");
        }
        start += 1;
    }
}

fn build_client(url: &str) -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_static(fomrat!("XSRF-TOKEN={XSR_TOKEN}}; osu_session={OSU_SESSION}")));
            headers.insert(reqwest::header::REFERER, reqwest::header::HeaderValue::from_str(url).unwrap());
            headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0"));
            headers.insert(reqwest::header::CONNECTION, reqwest::header::HeaderValue::from_static("keep-alive"));
            headers.insert(reqwest::header::ACCEPT_ENCODING, reqwest::header::HeaderValue::from_static("gzip, deflate, br"));
            headers.insert(reqwest::header::ACCEPT, reqwest::header::HeaderValue::from_static("*/*"));

            headers
        })
}

//429 Too Many Requests
async fn download_file(url: String) -> Result<reqwest::StatusCode, reqwest::Error> {
    tokio::time::sleep(Duration::from_secs(2)).await;

    let client = build_client(&url).build()?;
    let response = client.get(&url).send().await?;

    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Ok(response.status());
    }
    let status = response.status();

    if response.status() == 302 {
        let redir_url = response
            .headers() //always going to have a location if status is 302
            .get("location")
            .unwrap()
            .to_str()
            .unwrap();
        let redir_client = build_client(redir_url).build()?;
        let redir_response = redir_client.get(redir_url).send().await?;

        if redir_response.status().is_success() {
            let file_name = {
                let split_size = url.split("/").count();
                let num = url.split("/").nth(split_size - 2).unwrap();
                format!("out_maps/{num}.osz")
            };

            let mut file = std::fs::File::create(file_name).unwrap();
            let mut content = Cursor::new(redir_response.bytes().await?);

            std::io::copy(&mut content, &mut file).unwrap();
        }
    }
    Ok(status)
}
