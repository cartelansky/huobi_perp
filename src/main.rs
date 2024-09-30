use reqwest;
use serde_json::Value;
use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.hbdm.com/linear-swap-api/v1/swap_contract_info";
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(format!("API isteği başarısız: {}", response.status()).into());
    }

    let text = response.text().await?;
    let data: Value = serde_json::from_str(&text).map_err(|e| {
        eprintln!("JSON ayrıştırma hatası: {}", e);
        eprintln!("Alınan yanıt: {}", text);
        e
    })?;

    let mut coins: Vec<String> = Vec::new();

    if let Some(contracts) = data["data"].as_array() {
        for contract in contracts {
            if let (Some(symbol), Some(contract_status)) = (
                contract["contract_code"].as_str(),
                contract["contract_status"].as_u64(),
            ) {
                if symbol.ends_with("-USDT") && contract_status == 1 {
                    let coin = symbol.strip_suffix("-USDT").unwrap();
                    coins.push(format!("HUOBI:{}USDT.P", coin));
                }
            }
        }
    } else {
        return Err("API yanıtında beklenen 'data' alanı bulunamadı".into());
    }

    coins.sort_by(|a, b| {
        let a_symbol = a
            .strip_prefix("HUOBI:")
            .unwrap()
            .strip_suffix("USDT.P")
            .unwrap();
        let b_symbol = b
            .strip_prefix("HUOBI:")
            .unwrap()
            .strip_suffix("USDT.P")
            .unwrap();

        let a_num = a_symbol.chars().next().unwrap().to_digit(10);
        let b_num = b_symbol.chars().next().unwrap().to_digit(10);

        match (a_num, b_num) {
            (Some(a), Some(b)) if a == b => a_symbol.cmp(b_symbol),
            (Some(a), Some(b)) => b.cmp(&a),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => a_symbol.cmp(b_symbol),
        }
    });

    let mut file = File::create("huobi_usdt_perpetual_futures.txt")?;
    for coin in coins {
        writeln!(file, "{}", coin)?;
    }

    println!("Veriler başarıyla 'huobi_usdt_perpetual_futures.txt' dosyasına yazıldı.");
    Ok(())
}
