use tokio;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use tokio::time::{sleep, Duration};
use std::time::{SystemTime};
use std::time::Instant;

const APP_USER_AGENT: &str = "Mozilla/5.0 (X11; OSX x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36";
#[actix_web::main]
async fn main() {
    let targets_file = "targets.txt";

    let connection = sqlite::open("/usr/shared/sites.db").unwrap();

    connection.execute(
        "\
CREATE TABLE IF NOT EXISTS sites (time INTEGER, site TEXT, code TEXT, elapsed REAL);
        "
    ).unwrap();

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(20))
        .build().unwrap();

    loop {
        let sites = read_lines(targets_file).unwrap();

        for site in sites.into_iter() {
            let url = site.unwrap();

            let now = Instant::now();

            let client_check = client.clone();

            let check_result = match client_check.get(url.clone()).send().await {
                Ok(code) => { format!("{}", code.status()) }
                Err(_) => { String::from("err") }
            };

            let elapsed = now.elapsed();

            let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
            let sql = format!("INSERT INTO sites VALUES ({},'{}','{}',{})",timestamp.as_secs(), url.clone(),check_result.clone(), elapsed.clone().as_secs_f32());
            connection.execute(sql).unwrap();

            println!("site: {}, code: {}, elapsed time: {}", url.clone(), check_result, elapsed.as_secs_f32());

        }

        connection
            .iterate("SELECT count(*), avg(elapsed), site FROM sites GROUP BY site", |pairs| {
                for &(column, value) in pairs.iter() {
                    println!("{} = {}", column, value.unwrap());
                }
                true
            })
            .unwrap();

        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();

        connection.execute(format!("DELETE from sites where time < {}", timestamp.as_secs() - 1800)).unwrap();
        sleep(Duration::from_secs(5)).await;
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
