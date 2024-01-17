use chrono::{Datelike, DateTime, Local};
use serde::Deserialize;
use error_chain::error_chain;
use colored::Colorize;


error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

#[derive(Debug, Deserialize)]
struct TimePris {
    // Define the fields of your object
    #[serde(alias = "NOK_per_kWh")]
    price: f32,
    #[serde(alias = "time_start")]
    valid_from: DateTime<Local>,
    #[serde(alias = "time_end")]
    valid_to: DateTime<Local>,
}


#[tokio::main]
async fn main() -> Result<()> {
    let now = Local::now();
    let url_today = format!("https://www.hvakosterstrommen.no/api/v1/prices/{}/{}-{}_NO3.json", now.year(), format!("{:02}", now.month()), format!("{:02}", now.day()));
    let tomorrow = now + chrono::Duration::days(1);
    let url_tomorrow = format!("https://www.hvakosterstrommen.no/api/v1/prices/{}/{}-{}_NO3.json", tomorrow.year(), format!("{:02}", tomorrow.month()), format!("{:02}", tomorrow.day()));

    let res = reqwest::get(url_today).await?;
    let today_prices = res.text().await?;
    let mut prices: Vec<TimePris> = serde_json::from_str(today_prices.as_str()).expect("JSON was not well-formatted");

    let res = reqwest::get(url_tomorrow).await?;
    let tomorrow_prices = res.text().await?;
    let mut tomorrow: Vec<TimePris> = serde_json::from_str(tomorrow_prices.as_str()).expect("JSON was not well-formatted");

    // merge today and tomorrow vecs
    prices.append(&mut tomorrow);

    let mut calc_prices = Vec::new();
    for hour in prices.iter() {
        if hour.valid_to > now {
            calc_prices.push(hour);
        }
    }

    for hour in calc_prices.iter().take(10) {
        println!("{} - {} - {}", hour.valid_from, hour.valid_to, hour.price);
    }

    // usize to float


    let avg = calc_prices.iter().map(|x| x.price).sum::<f32>() / calc_prices.len() as f32;

    if avg < 0.5 {
        println!("Average price the next 10 hours: {}", avg.to_string().bold().green());
    } else if avg < 0.8 {
        println!("Average price the next 10 hours: {}", avg.to_string().yellow());
    } else {
        println!("Average price the next 10 hours: {}", avg.to_string().bold().red());
    }

    Ok(())
}