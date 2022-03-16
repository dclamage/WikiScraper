use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::collections::BTreeSet;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut answer_words_set: BTreeSet<String> = BTreeSet::new();
    let mut all_words_set: BTreeSet<String> = BTreeSet::new();

    let client = Client::builder().cookie_store(true).build()?;

    let response = client.get("https://coppermind.net/wiki/Category:Cosmere").send()?;
    let body_response = response.text()?;
    let document = Html::parse_document(&body_response);
    let link_selector = Selector::parse("a").unwrap();
    let mut addresses = Vec::new();
    for link in document.select(&link_selector) {
        let link_address = link.value().attr("href");
        if let Some(link_address) = link_address {
            if !link_address.contains(':') && link_address.starts_with("/wiki/") {
                addresses.push(link_address);
            }
        }
    }

    let num_addresses = addresses.len();
    let mut cur_address = 0;
    for address in addresses {
        cur_address = cur_address + 1;

        let full_address = format!("https://coppermind.net{}", address);
        println!("{} / {} ({:.2}%): {}", cur_address, num_addresses, (cur_address as f32 / num_addresses as f32 * 100.0), full_address);

        let response = client.get(full_address).send()?;
        let body_response = response.text()?;
        let document = Html::parse_document(&body_response);
        let heading_selector = &Selector::parse("#firstHeading").expect("Error parsing #firstHeading");
        let heading_text: String = document
            .select(heading_selector)
            .flat_map(|el| el.text())
            .collect();

        let group_selector = &Selector::parse("table.infobox.side > tbody > tr.kv > td")
            .expect("Error parsing #Group > table > tbody > tr > td");
        let group_text: String = document
            .select(group_selector)
            .flat_map(|el| el.text())
            .collect();

        let paragraph_selector = &Selector::parse("p").expect("p");
        let paragraph_text: String = document
            .select(paragraph_selector)
            .flat_map(|el| el.text())
            .collect();

        // Split them into words
        let heading_words: Vec<&str> = heading_text.split_whitespace().collect();
        let group_words: Vec<&str> = group_text.split_whitespace().collect();
        let paragraph_words: Vec<&str> = paragraph_text.split_whitespace().collect();

        // Construct the list of answer words
        let answer_words: Vec<&str> = heading_words
            .into_iter()
            .chain(group_words.into_iter())
            .collect();
        let answer_words: Vec<String> = answer_words.iter().map(|word| fix_word(word)).collect();
        let mut answer_words: Vec<String> = answer_words
            .iter()
            .filter(|word| word.len() == 5)
            .map(|str| str.to_string())
            .collect();
        answer_words.sort();
        answer_words.dedup();
        answer_words_set.extend(answer_words);

        // Construct the full list of words
        let paragraph_words: Vec<String> = paragraph_words.iter().map(|word| fix_word(word)).collect();
        let mut paragraph_words: Vec<String> = paragraph_words
            .iter()
            .filter(|word| word.len() == 5)
            .map(|str| str.to_string())
            .collect();
        paragraph_words.sort();
        paragraph_words.dedup();
        all_words_set.extend(paragraph_words);
    }

    all_words_set.extend(answer_words_set.clone());

    // Write the sets to disk
    let mut answer_words_file = std::fs::File::create("answer_words.txt")?;
    let mut all_words_file = std::fs::File::create("all_words.txt")?;
    for word in answer_words_set.iter() {
        answer_words_file.write_all(word.as_bytes())?;
        answer_words_file.write_all(b"\n")?;
    }
    for word in all_words_set.iter() {
        all_words_file.write_all(word.as_bytes())?;
        all_words_file.write_all(b"\n")?;
    }

    Ok(())
}

fn fix_word(word: &str) -> String {
    let mut fixed_word = String::new();
    for c in word.chars() {
        let lc = c.to_ascii_lowercase();
        if lc >= 'a' && lc <= 'z' {
            fixed_word.push(lc);
        }
    }
    fixed_word
}
