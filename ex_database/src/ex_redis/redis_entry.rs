use ex_config::config_format;
use r2d2::{Builder, Pool};
use redis::ConnectionAddr::Tcp;
use redis::{Cmd, Connection, ConnectionInfo, ConnectionLike, RedisConnectionInfo, RedisError};

use crate::common::builder_entry::{self, Config};

pub struct RedisStub {
    connection_info_: redis::ConnectionInfo,
}

impl r2d2::ManageConnection for RedisStub {
    type Connection = redis::Connection;
    type Error = RedisError;

    fn connect(&self) -> anyhow::Result<Connection, Self::Error> {
        let client = redis::Client::open(self.connection_info_.clone())?;
        client.get_connection()
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> anyhow::Result<(), Self::Error> {
        let _ = conn.req_command(Cmd::new().arg("PING"))?;
        Ok(())
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        conn.is_open() == false
    }
}

pub type RedisPool = Pool<RedisStub>;
pub type StubConfig = Config<RedisStub>;
type FnStubBuildHook = Option<fn(&mut Builder<RedisStub>)>;

pub fn make_connection_info(
    ip: &str,
    port: u16,
    db_no: i64,
    username: Option<&str>,
    password: Option<&str>,
) -> ConnectionInfo {
    ConnectionInfo {
        addr: Tcp(ip.to_owned(), port),
        redis: RedisConnectionInfo {
            db: db_no,
            username: if let Some(username) = username {
                Some(username.to_owned())
            } else {
                None
            },
            password: if let Some(password) = password {
                Some(password.to_owned())
            } else {
                None
            },
        },
    }
}

pub fn make_connection_info_from_config(redis_conf: &config_format::RedisConfig) -> ConnectionInfo {
    let host = &redis_conf.host;
    make_connection_info(&host.ip[..], host.port, redis_conf.db_no, None, None)
}

pub fn make_pool_default(
    connnection_info: ConnectionInfo,
    config: StubConfig,
    fn_build_hook: FnStubBuildHook,
) -> anyhow::Result<Pool<RedisStub>> {
    let mut builder = builder_entry::make_configured_builder::<RedisStub>(config);

    // hooking
    if fn_build_hook.is_none() == false {
        fn_build_hook.unwrap()(&mut builder);
    }

    let stub = RedisStub {
        connection_info_: connnection_info,
    };
    let pool = builder.build(stub)?;
    Ok(pool)
}
