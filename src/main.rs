use std::thread;
use std::time::Duration;
use clipboard_win::{formats, get_clipboard};
use std::fs;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::error::Error;
use serde::Deserialize;
use std::io;
use simulate;

fn main() {
    let mut input: String = String::new();
    while input.trim() != String::from("y") && input.trim() != String::from("n") {
        println!("Remplacer le texte par sa traduction automatiquement ? (y/n)");
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
    }
    let do_replace_text: bool;
    if input.trim() == "y" {
        do_replace_text = true;
    }
    else {
        do_replace_text = false;
    }

    let api_key: String = get_api_key();

    let mut selected_word: String = get_clipboard(formats::Unicode).expect("Please copy something before running the program");
    let mut word: String = selected_word.clone();

    loop {
        thread::sleep(Duration::from_millis(10));

        selected_word = get_clipboard(formats::Unicode).expect("Please copy something before running the program");

        if selected_word != word {
            if selected_word == String::from("STOP") {
                break;
            }
            word = selected_word.clone();
            let translation = get_translation(&api_key, &word);
            if do_replace_text {
                replace_text(&word, &translation);
            } else {
                println!("{}", do_replace_text);
            }
            println!("{}", translation);
        }

    }
}

fn replace_text(word: &String, translation: &String) {
    let mut text: String = word.to_owned();
    text += " (";
    text.push_str(&translation);
    text += ") ";
    simulate::type_str(&text).unwrap();
}

fn get_api_key() -> String {
    let key = fs::read_to_string("api/api_key.txt")
        .expect("Should have been able to read the file");

    key
}

fn get_translation(auth: &str, word: &str) -> String {

    #[derive(Debug, Deserialize)]
    struct Translation {
        detected_source_language: String,
        text: String,
    }

    #[derive(Debug, Deserialize)]
    struct Translations {
        translations: Vec<Translation>,
    }

    match send_translation_request(&auth, &word) {
        Ok(response_body) => {

            let translations: Translations = serde_json::from_str(&response_body).expect("Failed to parse JSON");

            if let Some(translation) = translations.translations.first() {
                return translation.text.clone();
            } else {
                println!("No translations found.");
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
    return String::from("An error has occured in get_translation()");
}

fn send_translation_request(auth: &str, word: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let auth_key = auth;

    let mut headers = HeaderMap::new();
    let auth_header_value = format!("DeepL-Auth-Key {}", auth_key);
    let auth_header = HeaderValue::from_str(&auth_header_value)?;

    headers.insert("Authorization", auth_header);
    headers.insert(USER_AGENT, HeaderValue::from_static("ubersetzer/1.0"));

    let params = vec![
        ("text", word),
        ("target_lang", "FR"),
    ];

    let response = client
        .post("https://api-free.deepl.com/v2/translate")
        .headers(headers)
        .form(&params)
        .send()?;

    // Check the response status
    if response.status().is_success() {
        let body = response.text()?;
        Ok(body)
    } else {
        Err("Request failed".into())
    }
}