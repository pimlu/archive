use std::fmt::Display;
use std::sync::mpsc;

use js_sys::ArrayBuffer;
use js_sys::Reflect;
use js_sys::Uint8Array;
use log::info;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    MessageEvent, Request, RequestInit, RequestMode, Response, RtcConfiguration, RtcDataChannel,
    RtcDataChannelType, RtcPeerConnection, RtcSdpType, RtcSessionDescriptionInit,
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
    pub rx: mpsc::Receiver<Vec<u8>>,
}
impl RtcClientSession for WasmClientSession {
    fn try_recv(&self) -> Result<Vec<u8>, mpsc::TryRecvError> {
        self.rx.try_recv()
    }
}

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

        dc.set_binary_type(RtcDataChannelType::Arraybuffer);

        let onclose_callback = Closure::wrap(Box::new(|| info!("closed")) as Box<dyn FnMut()>);
        let onopen_callback = Closure::wrap(Box::new(|| info!("open")) as Box<dyn FnMut()>);

        let (tx, rx) = mpsc::channel::<Vec<u8>>();

        // let dc_clone = dc.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |ev: MessageEvent| {
            let buffer = ev.data().dyn_into::<ArrayBuffer>();
            match buffer {
                Ok(buffer) => {
                    // convert it back into a jsvalue, we could have kept this
                    // originally but I'd rather make sure it's an arraybuffer
                    let vec = Uint8Array::new(&JsValue::from(buffer)).to_vec();
                    tx.send(vec);
                    // dc_clone.send_with_str("Pong from pc1.dc!").unwrap();
                }
                Err(e) => panic!("bad recv {:?}", e),
            }
        }) as Box<dyn FnMut(MessageEvent)>);
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
            rx,
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
