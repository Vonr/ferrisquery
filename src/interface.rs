use rcon::Result;
use uuid_mc::PlayerUuid;

type Connection = rcon::Connection<tokio::net::TcpStream>;

pub struct Interface {
    address: Box<str>,
    password: Box<str>,
    connection: Option<Connection>,
}

impl Interface {
    pub fn new(addr: impl ToString, pass: impl ToString) -> Self {
        Self {
            address: addr.to_string().into_boxed_str(),
            password: pass.to_string().into_boxed_str(),
            connection: None,
        }
    }

    pub async fn exec(&mut self, command: &str) -> Result<String> {
        if let Some(conn) = &mut self.connection {
            match conn.cmd(command).await {
                x @ Ok(..) | x @ Err(rcon::Error::CommandTooLong | rcon::Error::Auth) => return x,
                Err(rcon::Error::Io(..)) => {} // purposefully exhaustive for future-proofness
            }
        }

        // if we're here, either connection is None or it got disconnected - either way, we have to renew it
        self.renew_connection().await?.cmd(command).await
    }

    /// Both updates the internal connection and also returns it.
    async fn renew_connection(&mut self) -> Result<&mut Connection> {
        match Connection::connect(&*self.address, &self.password).await {
            Ok(conn) => Ok(self.connection.insert(conn)),
            Err(why) => Err(why),
        }
    }

    pub async fn player_list(&mut self) -> std::result::Result<Vec<PlayerInfo>, crate::Error> {
        use chumsky::prelude::*;

        let list_output = self.exec("list uuids").await?;

        fn parser<'a>() -> impl Parser<'a, &'a str, Vec<PlayerInfo>, extra::Err<Rich<'a, char>>> {
            let hex = |n| {
                any()
                    .filter(char::is_ascii_hexdigit)
                    .repeated()
                    .exactly(n)
                    .to_slice()
                    .try_map(|s: &str, span| {
                        u128::from_str_radix(s, 16).map_err(|e| Rich::custom(span, e))
                    })
            };

            group((
                just("There are ").ignored(),
                text::int(10).ignored(),
                just(" of a max ").ignored(),
                text::int(10).ignored(),
                just(" players online: ").ignored(),
                group((
                    any()
                        .filter(|b| *b != ' ')
                        .repeated()
                        .at_least(1)
                        .to_slice(),
                    just(' ').ignored(),
                    group((
                        hex(8),
                        just('-').ignored(),
                        hex(4),
                        just('-').ignored(),
                        hex(4),
                        just('-').ignored(),
                        hex(4),
                        just('-').ignored(),
                        hex(12),
                    ))
                    .try_map(|(a, _, b, _, c, _, d, _, e), span| {
                        let mut uuid = a;
                        uuid <<= 16;
                        uuid += b;
                        uuid <<= 16;
                        uuid += c;
                        uuid <<= 16;
                        uuid += d;
                        uuid <<= 48;
                        uuid += e;

                        PlayerUuid::new_with_uuid(uuid_mc::Uuid::from_u128(uuid))
                            .map_err(|e| Rich::custom(span, e))
                    })
                    .delimited_by(just('('), just(')')),
                ))
                .map(|(name, _, uuid): (&str, (), _)| PlayerInfo {
                    name: name.into(),
                    uuid,
                })
                .separated_by(just(' '))
                .collect::<Vec<_>>(),
            ))
            .map(|(_, _, _, _, _, info)| info)
        }

        let result = parser().parse(&list_output).into_result();
        result.map_err(|e| format!("Failed to parse output of `/list uuids`: {:?}", e).into())
    }
}

#[derive(Clone)]
pub struct PlayerInfo {
    pub name: Box<str>,
    pub uuid: uuid_mc::PlayerUuid,
}
