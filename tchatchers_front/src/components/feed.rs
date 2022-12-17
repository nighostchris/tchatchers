use std::rc::Rc;

// Copyright ⓒ 2022 LABEYE Loïc
// This tool is distributed under the MIT License, check out [here](https://github.com/nag763/tchatchers/blob/main/LICENSE.MD).
use super::chat::Chat;
use super::disconnected_bar::DisconnectedBar;
use super::type_bar::TypeBar;
use crate::components::toast::Alert;
use crate::router::Route;
use crate::services::chat_bus::ChatBus;
use crate::services::chat_service::WebsocketService;
use crate::services::toast_bus::ToastBus;
use crate::utils::jwt::get_user;
use gloo_net::http::Request;
use gloo_timers::callback::{Interval, Timeout};
use tchatchers_core::user::PartialUser;
use tchatchers_core::ws_message::{WsMessage, WsMessageContent};
use uuid::Uuid;
use yew::{html, Callback, Component, Context, Html, Properties};
use yew_agent::Dispatched;
use yew_agent::{Bridge, Bridged};
use yew_router::scope_ext::RouterScopeExt;

#[derive(Clone)]
pub enum Msg {
    HandleWsInteraction(Box<WsMessage>),
    CheckWsState,
    TryReconnect,
    CutWs,
}

#[derive(Clone, Eq, PartialEq, Properties)]
pub struct Props {
    pub room: String,
}

pub struct Feed {
    received_messages: Vec<WsMessageContent>,
    ws: WebsocketService,
    _producer: Box<dyn Bridge<ChatBus>>,
    _first_connect: Timeout,
    called_back: bool,
    is_connected: bool,
    ws_keep_alive: Option<Interval>,
    is_closed: bool,
    user: PartialUser,
    session_id: Uuid,
}

impl Component for Feed {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let ws: WebsocketService = WebsocketService::new(&ctx.props().room);
        let link = ctx.link().clone();
        let user = match get_user() {
            Ok(v) => v,
            Err(e) => {
                gloo_console::log!("Error while attempting to get the user :", e);
                link.navigator().unwrap().push(&Route::SignIn);
                ToastBus::dispatcher().send(Alert {
                    is_success: false,
                    content: "Please authenticate prior accessing the app functionnalities.".into(),
                });
                PartialUser::default()
            }
        };
        let cb = {
            let link = ctx.link().clone();
            move |msg| link.send_message(Msg::HandleWsInteraction(Box::new(msg)))
        };
        Self {
            received_messages: vec![],
            ws,
            _producer: ChatBus::bridge(Rc::new(cb)),
            is_connected: false,
            called_back: false,
            is_closed: false,
            ws_keep_alive: None,
            user,
            _first_connect: {
                let link = ctx.link().clone();
                Timeout::new(1, move || link.send_message(Msg::CheckWsState))
            },
            session_id: Uuid::new_v4(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleWsInteraction(message) => {
                self.called_back = true;
                match *message {
                    WsMessage::ClientDisconnected | WsMessage::ConnectionClosed => {
                        if !self.is_closed {
                            gloo_console::error!("Not connected");
                            let req = Request::get("/api/validate").send();
                            let link = ctx.link().clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                let resp = req.await.unwrap();
                                if resp.status() == 401 {
                                    link.navigator().unwrap().push(&Route::SignIn);
                                }
                            });
                            self.ws_keep_alive = None;
                            self.is_connected = false;
                        }
                    }
                    WsMessage::Receive(msg_content) => {
                        self.received_messages.insert(0, msg_content)
                    }
                    WsMessage::MessagesRetrieved {
                        mut messages,
                        session_id,
                    } if session_id == self.session_id => {
                        self.received_messages.append(&mut messages)
                    }
                    WsMessage::Pong => {
                        self.is_connected = true;

                        if self.received_messages.is_empty() {
                            let msg = WsMessage::RetrieveMessages(self.session_id);
                            self.ws
                                .tx
                                .clone()
                                .try_send(serde_json::to_string(&msg).unwrap())
                                .unwrap();
                            self.ws_keep_alive = {
                                let tx = self.ws.tx.clone();
                                Some(Interval::new(30_000, move || {
                                    tx.clone().try_send("Keep Alive".into()).unwrap()
                                }))
                            }
                        }
                    }
                    _ => {
                        self.is_connected = true;
                    }
                }
                true
            }
            Msg::CheckWsState => {
                self.ws
                    .tx
                    .clone()
                    .try_send(serde_json::to_string(&WsMessage::Ping).unwrap())
                    .unwrap();
                false
            }
            Msg::TryReconnect => {
                let ws: WebsocketService = WebsocketService::new(&ctx.props().room);
                self.ws = ws;
                self.ws
                    .tx
                    .clone()
                    .try_send(serde_json::to_string(&WsMessage::Ping).unwrap())
                    .unwrap();
                self.called_back = false;
                true
            }
            Msg::CutWs => {
                self.is_closed = true;
                let mut ws = self.ws.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    ws.close().await;
                });
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let component: Html = match self.is_connected {
            true => {
                let tx = self.ws.tx.clone();
                let pass_message_to_ws = Callback::from(move |message: String| {
                    tx.clone().try_send(message).unwrap();
                });
                html! {<TypeBar {pass_message_to_ws} user={self.user.clone()} room={ctx.props().room.clone()}/>}
            }
            false => {
                let link = ctx.link().clone();
                let try_reconnect = Callback::from(move |_: ()| {
                    link.send_message(Msg::TryReconnect);
                });
                html! {<DisconnectedBar called_back={self.called_back} {try_reconnect} />}
            }
        };
        html! {
            <div class="grid grid-rows-11 h-full dark:bg-zinc-800">
                <div class="row-span-10 overflow-auto flex flex-col-reverse" >
                    <Chat messages={self.received_messages.clone()} room={ctx.props().room.clone()} user={self.user.clone()} />
                </div>
                <div class="row-span-1 grid grid-cols-6 px-5 gap-4 justify-center content-center block">
                    {component}
                </div>
            </div>
        }
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        self.ws
            .tx
            .try_send(serde_json::to_string(&WsMessage::Close).unwrap())
            .unwrap();
        ctx.link().send_message(Msg::CutWs)
    }
}
