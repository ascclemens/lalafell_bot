use database::schema::*;
use database::models::U64;

use byteorder::{ByteOrder, LittleEndian};

insertable! {
  #[derive(Debug, Queryable, Identifiable)]
  #[table_name = "delete_all_messages"]
  pub struct DeleteAllMessages,
  #[derive(Debug, Insertable)]
  #[table_name = "delete_all_messages"]
  pub struct NewDeleteAllMessages {
    pub server_id: U64,
    pub channel_id: U64,
    pub after: i32,
    pub exclude: Vec<u8>
  }
}

impl DeleteAllMessages {
  pub fn exclude(&self) -> Vec<u64> {
    self.exclude.chunks(8).map(|x| LittleEndian::read_u64(x)).collect()
  }
}

impl NewDeleteAllMessages {
  pub fn new(server_id: u64, channel_id: u64, after: i32, exclude: &[u64]) -> Self {
    let mut bytes = vec![0; exclude.len() * 8];
    if !exclude.is_empty() {
      LittleEndian::write_u64_into(exclude, &mut bytes);
    }
    NewDeleteAllMessages {
      server_id: server_id.into(),
      channel_id: channel_id.into(),
      after,
      exclude: bytes
    }
  }
}
