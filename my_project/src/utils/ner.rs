//use reqwest::Client;
//use serde_json::Value;

/// Calls Python NER API and extracts entities
//pub async fn call_ner_api(text: &str) -> Result<Vec<(String, String)>, reqwest::Error> {
  //  let client = Client::new();
    //let res: Value = client
    //    .post("https://YOUR-NGROK-ENDPOINT.ngrok-free.app/ner")
    //    .json(&serde_json::json!({ "text": text }))
       // .send()
        //.await?
        //.json()
       // .await?;

  //  let mut entities = Vec::new();
   // if let Some(entity_list) = res["entities"].as_array() {
       // for entity in entity_list {
          //  if let (Some(text), Some(label)) = (entity[0].as_str(), entity[1].as_str()) {
               // entities.push((text.to_string(), label.to_string()));
          //  }
       // }
   // }

   // Ok(entities)
//}