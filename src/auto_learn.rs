use reqwest;
use scraper::{Html, Selector};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use urlencoding::encode;

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .user_agent("PappapAIChain-SNN/0.3 (+https://pappap.ai)")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap()
});

static KNOWLEDGE_DB: Lazy<Arc<RwLock<HashMap<String, String>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub async fn auto_learn_and_answer(question: &str) -> String {
    let key = question.to_lowercase();

    // Kiểm tra đã học chưa
    {
        let db = KNOWLEDGE_DB.read().await;
        if let Some(answer) = db.get(&key) {
            return format!("[ĐÃ HỌC TRƯỚC] {answer}");
        }
    }

    // Xây dựng query – tất cả đều là String
    let query: String = if question.contains("luật giao thông") || question.contains("Luật") {
        "Luật giao thông đường bộ Việt Nam 2025 site:thuvienphapluat.vn OR site:luatvietnam.vn".to_string()
    } else if question.contains("bài tập") || question.contains("giải") {
        format!("{} giải chi tiết site:loigiaihay.com OR site:violet.vn OR site:mathvn.com", question)
    } else {
        format!("{} site:vi.wikipedia.org OR site:vnexpress.net OR site:tuoitre.vn OR site:thuvienphapluat.vn", question)
    };

    let google_url = format!("https://www.google.com/search?q={}", encode(&query));

    let body = match CLIENT.get(&google_url).send().await {
        Ok(r) => match r.text().await {
            Ok(t) => t,
            Err(_) => return "Pappap đang học… nhưng mạng hơi chậm ạ!".to_string(),
        },
        Err(_) => return "Pappap đang học… nhưng mạng hơi chậm ạ!".to_string(),
    };

    let document = Html::parse_document(&body);
    let selector = Selector::parse("a").unwrap();
    let mut urls = Vec::new();

    for element in document.select(&selector).take(10) {
        if let Some(href) = element.value().attr("href") {
            if href.contains("/url?q=") {
                if let Some(real_url) = href.split('&').next()
                    .and_then(|s| s.strip_prefix("/url?q=")) 
                {
                    let real_url_str = real_url.to_string();
                    if real_url_str.contains("thuvienphapluat.vn") ||
                       real_url_str.contains("luatvietnam.vn") ||
                       real_url_str.contains("loigiaihay.com") ||
                       real_url_str.contains("vnexpress.net") ||
                       real_url_str.contains("tuoitre.vn") ||
                       real_url_str.contains("violet.vn") {
                        urls.push(real_url_str);
                    }
                }
            }
        }
    }

    let mut learned_text = String::new();
    for url in urls.iter().take(3) {
        match CLIENT.get(url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    let frag = Html::parse_document(&text);
                    let p_sel = Selector::parse("p, div, article, li, span").unwrap();
                    let content: String = frag.select(&p_sel)
                        .filter_map(|e| e.text().next())
                        .collect::<Vec<_>>()
                        .join(" ")
                        .chars()
                        .take(2000)
                        .collect();

                    if content.len() > 300 {
                        learned_text = content;
                        break;
                    }
                }
                Err(_) => continue,
            },
            Err(_) => continue,
        }
    }

    if learned_text.is_empty() {
        return "Pappap chưa tìm được tài liệu đáng tin cậy, để em học thêm rồi trả lời sau nha!".to_string();
    }

    // Lưu vào bộ nhớ dài hạn
    {
        let mut db = KNOWLEDGE_DB.write().await;
        db.insert(key, learned_text.clone());
    }

    format!("Pappap vừa tự học mới!\n\n{learned_text}\n\n→ Đã ghi nhớ vĩnh viễn kiến thức này!")
}