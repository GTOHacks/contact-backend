use chrono::{
	DateTime,
	Utc,
};

use std::{
	env,
	time::SystemTime,
	convert::Infallible,
	net::SocketAddr,
};

use http_body_util::{
	BodyExt,
	Full,
};

use hyper::{
	body::{
		Body,
		Bytes,
	},
	server::conn::http1,
	service::service_fn,
	header::HeaderValue,
	Request,
	Response,
	Method,
	StatusCode,
};

use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use serde_json::{
	Value,
	Value::Null,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let addr = SocketAddr::from(([127, 0, 0, 1], 6980));

	let listener = TcpListener::bind(addr).await?;

	loop {
		let (stream, addr) = listener.accept().await?;

		let io = TokioIo::new(stream);

		let service = service_fn(move |req| {
			let addr = addr.clone();
			async move {
				match serve(req, addr).await {
					Ok(resp) => Ok::<Response<Full<Bytes>>, Infallible>(resp),
					Err(e) => {
						println!("Error in serve: {:?}", e);
						let mut resp = Response::new(Full::new(Bytes::from("Internal Server Error")));
						*resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
						return Ok(resp);
					}
				}
			}
		});

		tokio::task::spawn(async move {
			if let Err(err) = http1::Builder::new()
				.serve_connection(io, service)
				.await {
					println!("Error: {:?}", err);
				}
		});
	}
}

async fn serve(req: Request<hyper::body::Incoming>, addr: SocketAddr) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error>> {
	match (req.method(), req.uri().path()) {
		(&Method::POST, "/contact") => {
			let max = req.body().size_hint().upper().unwrap_or(u64::MAX);
			if max > 65536 {
				let mut resp = Response::new(Full::new(Bytes::from("Request too long")));
				*resp.status_mut() = StatusCode::PAYLOAD_TOO_LARGE;
				return Ok(resp);
			}

			let body = &String::from_utf8(req.collect().await?.to_bytes().to_vec()).unwrap();

			let data: Value = match serde_json::from_str(body) {
				Ok(data) => data,
				Err(e) => {
					println!("Error in from_slice: {:?}", e);
					let mut resp = Response::new(Full::new(Bytes::from("Bad Request")));
					*resp.status_mut() = StatusCode::BAD_REQUEST;
					return Ok(resp);
				}
			};

			match post_comment(data, &format!("{addr}")).await {
				Ok(resp) => return Ok(resp),
				Err(e) => {
					println!("Error in post_comment: {:?}", e);
					let mut resp = Response::new(Full::new(Bytes::from("Bad Gateway")));
					*resp.status_mut() = StatusCode::BAD_GATEWAY;
					return Ok(resp);
				},
			};
		},
		_ => {
			let mut resp = Response::new(Full::new(Bytes::from("")));
			resp.headers_mut().insert("Location", HeaderValue::from_static("https://gtohacks.github.io/website"));
			*resp.status_mut() = StatusCode::FOUND;
			Ok(resp)
		},
	}
}

async fn post_comment(data: Value, ipaddr: &str) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error>> {
	let uri: &str = &env::var("WH_URI")?;

	let now = SystemTime::now();
	let now: DateTime<Utc> = now.into();
	let now: &str = &now.to_rfc3339();
	
	if data["name"] == Null || data["title"] == Null || data["email"] == Null || data["message"] == Null {
		let mut resp = Response::new(Full::new(Bytes::from("Bad Request")));
		*resp.status_mut() = StatusCode::BAD_REQUEST;
		return Ok(resp);
	}

	let name: &str = data["name"].as_str().unwrap();
	let title: &str = data["title"].as_str().unwrap();
	let email: &str = data["email"].as_str().unwrap();
	let message: &str = data["message"].as_str().unwrap();

	let params = &serde_json::json!({
		"content": "new message!!! (totally)",
		"username": name,
		"embeds": [
			{
				"title": title,
				"color": 0x2b2d31,
				"fields": [
					{
						"name": "Name",
						"value": name
					},
					{
						"name": "Email",
						"value": email,
					},
					{
						"name": "IP Address",
						"value": format!("||{ipaddr}||"),
					},
				],
				"description": message,
				"timestamp": now,
			},
		],
	});

	let client = reqwest::Client::new();

	let wh_resp = client.post(uri)
		.json(params)
		.send()
		.await?;
	let status = wh_resp.status();
	let body = wh_resp.bytes().await?;
	let mut resp = Response::new(Full::new(Bytes::from(body)));
	*resp.status_mut() = StatusCode::from_u16(status.into())?;
	Ok(resp)
}
