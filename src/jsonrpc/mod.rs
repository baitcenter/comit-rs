extern crate reqwest;
extern crate serde;
extern crate serde_json;

use self::reqwest::Client as HTTPClient;
use self::serde::ser::{Serialize, SerializeSeq, Serializer};
use self::serde_json::Value;
use std::collections::HashMap;
use std::string::String;

struct JsonRpcClient {
    client: HTTPClient,
    url: String,
}

#[derive(Serialize)]
struct Payload<T> where T : Serialize {
    jsonrpc: String,
    id: String,
    method: String,
    params: T,
}

impl JsonRpcClient {
    fn new(client: HTTPClient, url: &str) -> Self {
        JsonRpcClient {
            client,
            url: url.to_string(),
        }
    }

    fn call<T>(&self, id: &str, method: &str, t: T) where T : Serialize {
        let payload = Payload {
            jsonrpc: "1.0".to_string(),
            id: id.to_string(),
            method: method.to_string(),
            params: t,
        };

        let response = self.client
            .post(self.url.as_str())
            .json(&payload)
            .send()
            .unwrap();

        println!("{:?}", response)
    }

    pub fn call1<A>(&self, id: &str, method: &str, a: A) where A : Serialize {
        self.call(id, method, [a])
    }

    pub fn call2<A, B>(&self, id: &str, method: &str, a: A, b: B) where A : Serialize, B : Serialize {
        self.call(id, method, (a, b))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_adssad() {
        let client = HTTPClient::new();
        let rpc_client = JsonRpcClient::new(client, "http://127.0.0.1:8080");

//        rpc_client.call1("id", "generate", 100);
        rpc_client.call2("id", "generate", 100, "adsasd");
    }
}