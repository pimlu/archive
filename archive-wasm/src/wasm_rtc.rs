use std::fmt::Display;

use js_sys::Reflect;
use log::info;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    MessageEvent, Request, RequestInit, RequestMode, Response, RtcConfiguration, RtcDataChannel,
    RtcPeerConnection, RtcSdpType, RtcSessionDescriptionInit,
};

use archive_engine::rtc::*;
use archive_engine::SharedFuture;

use wasm_bindgen::prelude::*;

pub fn fmt_jserr<T>(err: impl Display) -> Result<T, JsValue> {
    Err(JsValue::from(format!("{err}")))
}

pub struct WasmClientSession {
    pub peer_connection: RtcPeerConnection,
    pub data_channel: RtcDataChannel,
}
impl RtcClientSession for WasmClientSession {}

pub struct WasmServerHandle {
    pub hostname: String,
}
impl WasmServerHandle {
    async fn rtc_signal(
        hostname: String,
        client_offer: ClientOffer,
    ) -> Result<ServerAnswer, JsValue> {
        let offer_serialized = serde_json::to_string(&client_offer).or_else(fmt_jserr)?;

        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::Cors);
        opts.body(Some(&JsValue::from_str(&offer_serialized)));

        let url = format!("{hostname}/signal");

        let request = Request::new_with_str_and_init(&url, &opts)?;

        let headers = request.headers();
        headers.set("Content-Type", "application/json")?;
        headers.set("Accept", "application/json")?;

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

        // `resp_value` is a `Response` object.
        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into()?;

        // Convert this other `Promise` into a rust `Future`.
        let json = JsFuture::from(resp.json()?).await?;

        // Use serde to parse the JSON into a struct.
        let server_answer: ServerAnswer = json.into_serde().or_else(fmt_jserr)?;
        Ok(server_answer)
    }

    async fn rtc_connect_raw(hostname: String) -> Result<WasmClientSession, JsValue> {
        // based off of https://jsfiddle.net/9tsx15mg/90/ and the webrtc samples
        let mut config = RtcConfiguration::new();
        let ice_servers = js_sys::JSON::parse("[{\"urls\":\"stun:stun.l.google.com:19302\"}]")?;
        config.ice_servers(&ice_servers);
        let pc = RtcPeerConnection::new_with_configuration(&config)?;

        let dc = pc.create_data_channel("foo");

        let onclose_callback = Closure::wrap(Box::new(|| info!("closed")) as Box<dyn FnMut()>);
        let onopen_callback = Closure::wrap(Box::new(|| info!("open")) as Box<dyn FnMut()>);

        // let dc_clone = dc.clone();
        let onmessage_callback =
            Closure::wrap(
                Box::new(move |ev: MessageEvent| match ev.data().as_string() {
                    Some(message) => {
                        info!("message: {message}");
                        // dc_clone.send_with_str("Pong from pc1.dc!").unwrap();
                    }
                    None => {}
                }) as Box<dyn FnMut(MessageEvent)>,
            );
        dc.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        dc.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        dc.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();
        onopen_callback.forget();
        onmessage_callback.forget();

        // this was done in onnegotiationneeded in the fiddle, but the webrtc samples
        // don't bother and just do it directly during construction, which is easier
        let offer = JsFuture::from(pc.create_offer()).await?;

        let offer_sdp = Reflect::get(&offer, &JsValue::from_str("sdp"))?
            .as_string()
            .ok_or(JsValue::from("bad sdp"))?;

        let client_offer = ClientOffer { sdp: offer_sdp };

        let mut offer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
        offer_obj.sdp(&client_offer.sdp);
        let sld_promise = pc.set_local_description(&offer_obj);
        JsFuture::from(sld_promise).await?;

        // fetch the server's answer SDP and use it.
        let server_answer = WasmServerHandle::rtc_signal(hostname, client_offer).await?;

        let mut answer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
        answer_obj.sdp(&server_answer.sdp);
        let srd_promise = pc.set_remote_description(&answer_obj);
        JsFuture::from(srd_promise).await?;

        let session = WasmClientSession {
            peer_connection: pc,
            data_channel: dc,
        };
        Ok(session)
    }
}

impl RtcServerDescriptor for WasmServerHandle {
    type Session = WasmClientSession;

    type Error = JsValue;

    fn rtc_connect(&self) -> SharedFuture<Result<Self::Session, Self::Error>> {
        Box::pin(Self::rtc_connect_raw(self.hostname.clone()))
    }
}
