use serenity::{
  client::{Context, EventHandler},
  model::gateway::Ready,
};

pub struct RandomPresenceListener;

impl EventHandler for RandomPresenceListener {
  fn ready(&self, ctx: Context, _: Ready) {
    if let Some(g) = crate::tasks::random_presence::random_activity() {
      ctx.shard.set_activity(Some(g));
    }
  }
}
