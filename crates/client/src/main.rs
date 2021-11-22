use common::*;
use iced::button::{self, Button};
use iced::scrollable::{self, Scrollable};
use iced::text_input::{self, TextInput};
use iced::{Application, Color, Column, Command, Container, Element, Length, Row, Settings, Text};
use std::net::UdpSocket;

pub enum App {
    Loading {
        from_port_value: String,
        from_port_input: text_input::State,
        from_ip_value: String,
        from_ip_input: text_input::State,
        port_value: String,
        port_input: text_input::State,
        ip_value: String,
        ip_input: text_input::State,
        button: button::State,
        err: String,
    },
    Loaded(State),
}

pub struct State {
    scroll: scrollable::State,
    add_button: button::State,
    fetch_button: button::State,

    contacts: Vec<Contact>,
    name_value: String,
    number_value: String,
    number_input: text_input::State,
    input: text_input::State,
    socket: UdpSocket,
    addr: String,
}

pub struct Contact {
    name: String,
    number: String,
    state: ContactState,
    is_correct: bool,
}

#[derive(Debug, Clone)]
pub enum ContactState {
    Idle {
        edit_button: button::State,
    },
    Editing {
        number_input: text_input::State,
        delete_button: button::State,
    },
}

#[derive(Debug, Clone)]
pub enum ContactMessage {
    Edit,
    FinishEdition,
    Edited(String),
    Delete,
}
impl Contact {
    fn update(&mut self, message: ContactMessage) {
        match message {
            ContactMessage::Edit => {
                let text_input = text_input::State::focused();

                self.state = ContactState::Editing {
                    number_input: text_input,
                    delete_button: button::State::new(),
                };
            }
            ContactMessage::Delete => {}
            ContactMessage::Edited(number) => {
                self.number = number;
            }
            ContactMessage::FinishEdition => {
                if !self.number.is_empty()
                    && phonenumber::parse(None, &self.number)
                        .map(|x| phonenumber::is_valid(&x))
                        .unwrap_or_else(|_| false)
                {
                    self.is_correct = true;
                    self.state = ContactState::Idle {
                        edit_button: button::State::new(),
                    }
                } else {
                    self.is_correct = self.number.is_empty();
                }
            }
        }
    }

    fn view(&mut self) -> Element<ContactMessage> {
        match &mut self.state {
            ContactState::Idle { edit_button } => Row::new()
                .spacing(20)
                .push(
                    Text::new(&format!("{}: {}", self.name, self.number))
                        .horizontal_alignment(iced::HorizontalAlignment::Left),
                )
                .push(
                    Button::new(edit_button, Text::new("Edit"))
                        .on_press(ContactMessage::Edit)
                        .padding(10),
                )
                .align_items(iced::Align::Start)
                .into(),
            ContactState::Editing {
                number_input,
                delete_button,
            } => {
                let text_input = TextInput::new(
                    number_input,
                    "Enter new phone number",
                    &self.number,
                    ContactMessage::Edited,
                )
                .on_submit(ContactMessage::FinishEdition)
                .padding(10);

                let mut row = Row::new()
                    .spacing(20)
                    .align_items(iced::Align::Center)
                    .push(text_input)
                    .push(
                        Button::new(
                            delete_button,
                            Row::new()
                                .spacing(10)
                                .push(Text::new("Delete").color(Color::from_rgb(1.0, 0.0, 0.0))),
                        )
                        .on_press(ContactMessage::Delete)
                        .padding(10),
                    );
                if !self.is_correct {
                    row = row.push(
                        Text::new("Incorrect phone number").color(Color::from_rgb(1.0, 0.0, 0.0)),
                    );
                }
                row.into()
            }
        }
    }
}

impl Default for ContactState {
    fn default() -> Self {
        Self::Idle {
            edit_button: button::State::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    InputChanged2(String),
    InputChanged3(String),
    InputChanged4(String),
    Continue,
    ContactMessage(usize, ContactMessage),
    AddUser,
    GetAllUsers,
    DeleteUsers,
}

fn empty_message<'a>(message: &str) -> Element<'a, Message> {
    Container::new(
        Text::new(message)
            .width(Length::Fill)
            .size(25)
            .horizontal_alignment(iced::HorizontalAlignment::Center)
            .color([0.7, 0.7, 0.7]),
    )
    .width(Length::Fill)
    .height(Length::Units(200))
    .center_y()
    .into()
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self::Loading {
                ip_input: text_input::State::new(),
                ip_value: String::new(),
                port_input: text_input::State::new(),
                port_value: String::new(),
                button: button::State::new(),
                err: String::new(),
                from_ip_input: text_input::State::new(),
                from_ip_value: String::new(),
                from_port_input: text_input::State::new(),
                from_port_value: String::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Phone numbers".to_owned()
    }
    fn update(
        &mut self,
        message: Self::Message,
        _: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        match self {
            Self::Loaded(state) => match message {
                Message::InputChanged(input) => {
                    state.name_value = input;
                }
                Message::InputChanged2(input) => {
                    state.number_value = input;
                }
                Message::AddUser => {
                    let instruction = serde_json::to_vec(&Instruction::AddPhoneNumber {
                        key: state.name_value.clone(),
                        number: state.number_value.clone(),
                    })
                    .unwrap();
                    if state.contacts.iter().any(|x| x.name == state.name_value) {
                        return Command::none();
                    }
                    state.name_value.clear();
                    state.number_value.clear();
                    state.socket.send_to(&instruction, &state.addr).unwrap();
                    state.contacts.push(Contact {
                        state: ContactState::Idle {
                            edit_button: button::State::new(),
                        },
                        is_correct: true,
                        name: state.name_value.clone(),
                        number: state.number_value.clone(),
                    });
                }
                Message::ContactMessage(i, ContactMessage::Delete) => {
                    if state.contacts.len() > i {
                        let contact = state.contacts.remove(i);
                        let instruction =
                            serde_json::to_vec(&Instruction::DeleteUser { key: contact.name })
                                .unwrap();
                        state.socket.send_to(&instruction, &state.addr).unwrap();
                    }
                }
                Message::ContactMessage(i, message) => {
                    if let Some(contact) = state.contacts.get_mut(i) {
                        contact.update(message);
                    }
                }
                Message::GetAllUsers => {
                    let instruction = serde_json::to_vec(&Instruction::GetAllUsers).unwrap();
                    state.socket.send_to(&instruction, &state.addr).unwrap();

                    let mut vec = vec![0u8; 8 * 1024];

                    let (bytes, _) = state.socket.recv_from(&mut vec).unwrap();

                    let contacts = serde_json::from_slice::<Response>(&vec[..bytes]);
                    match contacts {
                        Ok(Response::AllUsers(contacts)) => {
                            state.contacts.clear();

                            for (contact, number) in contacts {
                                state.contacts.push(Contact {
                                    state: ContactState::Idle {
                                        edit_button: button::State::new(),
                                    },
                                    is_correct: true,
                                    name: contact.clone(),
                                    number: number.clone(),
                                })
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                _ => (),
            },
            Self::Loading {
                port_value,
                port_input: _,
                ip_value,
                button: _,
                ip_input: _,
                from_ip_input: _,
                from_ip_value,
                from_port_input: _,
                from_port_value,
                err,
            } => match message {
                Message::InputChanged3(ip) => {
                    *ip_value = ip;
                    err.clear();
                }
                Message::InputChanged4(port) => {
                    *port_value = port;
                    err.clear();
                }
                Message::InputChanged(ip) => {
                    *from_ip_value = ip;
                    err.clear();
                }
                Message::InputChanged2(port) => {
                    *from_port_value = port;
                    err.clear();
                }

                Message::Continue => {
                    let socket = UdpSocket::bind(format!("{}:{}", from_ip_value, from_port_value));
                    let socket = match socket {
                        Ok(x) => x,
                        Err(e) => {
                            *err = format!(
                                "Failed to bind socket to `{}:{}`: {}",
                                from_ip_value, from_port_value, e
                            );
                            return Command::none();
                        }
                    };
                    match socket.connect(format!("{}:{}", ip_value, port_value)) {
                        Ok(_) => {
                            *self = Self::Loaded(State {
                                add_button: button::State::new(),
                                input: text_input::State::new(),
                                scroll: scrollable::State::new(),
                                name_value: "".to_string(),
                                number_input: text_input::State::new(),
                                number_value: "".to_string(),
                                contacts: vec![],
                                fetch_button: button::State::new(),
                                socket,
                                addr: format!("{}:{}", ip_value, port_value),
                            })
                        }
                        Err(e) => {
                            *err = format!(
                                "Failed to connect socket to `{}:{}`: {}",
                                ip_value, port_value, e
                            );
                        }
                    }
                }
                _ => (),
            },
        }
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let title = Text::new("phone numbers")
            .width(Length::Fill)
            .size(50)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(iced::HorizontalAlignment::Center);
        match self {
            Self::Loaded(state) => {
                let contact_name = TextInput::new(
                    &mut state.input,
                    "User name",
                    &state.name_value,
                    Message::InputChanged,
                )
                .padding(15)
                .size(30);
                let mut content = Column::new();
                let exists = state.contacts.iter().any(|x| x.name == state.name_value);

                let parsed = phonenumber::parse(None, &state.number_value)
                    .map(|x| phonenumber::is_valid(&x))
                    .unwrap_or_else(|_| false);

                let number = TextInput::new(
                    &mut state.number_input,
                    "Phone number",
                    &state.number_value,
                    Message::InputChanged2,
                )
                .padding(15)
                .size(30);
                let contacts: Element<_> = if !state.contacts.is_empty() {
                    state
                        .contacts
                        .iter_mut()
                        .enumerate()
                        .fold(Column::new().spacing(20), |column, (i, contact)| {
                            column.push(
                                contact
                                    .view()
                                    .map(move |msg| Message::ContactMessage(i, msg)),
                            )
                        })
                        .into()
                } else {
                    empty_message("You do not have any contacts yet...")
                };

                content = content.max_width(640).spacing(20).push(title);
                if exists {
                    content = content.push(
                        Text::new("This user is already registered!")
                            .color(Color::from_rgb(1.0, 0.0, 0.0)),
                    );
                }
                content = content.push(contact_name).push(number);
                if !parsed && !state.number_value.is_empty() {
                    content = content.push(
                        Text::new("Invalid phone number").color(Color::from_rgb(1.0, 0.0, 0.0)),
                    );
                } else if parsed && !state.number_value.is_empty() && !state.name_value.is_empty() {
                    content = content.push(
                        Button::new(
                            &mut state.add_button,
                            Text::new("Add contact")
                                .horizontal_alignment(iced::HorizontalAlignment::Center),
                        )
                        .on_press(Message::AddUser),
                    );
                }
                content = content.push(
                    Button::new(
                        &mut state.fetch_button,
                        Text::new("Fetch contacts from server"),
                    )
                    .on_press(Message::GetAllUsers),
                );
                content = content.push(contacts);
                Scrollable::new(&mut state.scroll)
                    .padding(40)
                    .push(Container::new(content).width(Length::Fill).center_x())
                    .into()
            }
            Self::Loading {
                from_ip_input,
                from_ip_value,
                from_port_input,
                from_port_value,
                ip_input,
                ip_value,
                port_input,
                port_value,
                err,
                button,
            } => {
                let from_ip = TextInput::new(
                    from_ip_input,
                    "Client IP",
                    from_ip_value,
                    Message::InputChanged,
                )
                .padding(15)
                .size(30);

                let from_port = TextInput::new(
                    from_port_input,
                    "Client port",
                    from_port_value,
                    Message::InputChanged2,
                )
                .padding(15)
                .size(30);
                let ip =
                    TextInput::new(ip_input, "Connect to IP", ip_value, Message::InputChanged3)
                        .padding(15)
                        .size(30);

                let port = TextInput::new(
                    port_input,
                    "Connect to port",
                    port_value,
                    Message::InputChanged4,
                )
                .padding(15)
                .size(30);
                let continue_btn = Button::new(button, Text::new("Connect to phone numbers DB"))
                    .on_press(Message::Continue)
                    .padding(10);
                let mut content = Column::new()
                    .max_width(300)
                    .spacing(20)
                    .push(title)
                    .push(from_ip)
                    .push(from_port)
                    .push(Text::new("Enter IP & port of the server: "))
                    .push(ip)
                    .push(port)
                    .push(continue_btn);
                if !err.is_empty() {
                    content = content.push(
                        Text::new(format!("Error: {}", err)).color(Color::from_rgb(1.0, 0.0, 0.0)),
                    );
                }
                Container::new(content)
                    .width(Length::Fill)
                    .center_x()
                    .into()
            }
        }
    }
}

fn main() {
    App::run(Settings::default()).unwrap();
}
