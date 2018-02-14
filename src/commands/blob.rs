use lalafell::commands::prelude::*;

use std::collections::HashMap;

#[derive(BotCommand)]
pub struct BlobCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "Have the bot post a blob emoji for you")]
pub struct Params {
  #[structopt(help = "The name of the blob emoji")]
  name: String
}

impl HasParams for BlobCommand {
  type Params = Params;
}

impl<'a> Command<'a> for BlobCommand {
  fn run(&self, _: &Context, msg: &Message, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("blob", params, |a| a.setting(::structopt::clap::AppSettings::ArgRequiredElseHelp))?;

    if !BLOBS.contains_key(&params.name.as_str()) {
      return Err("That's not a valid blob name.".into());
    }
    let blob = BLOBS[params.name.as_str()];

    let name = msg.guild()
      .and_then(|guild| guild.read().members.get(&msg.author.id).map(|a| a.display_name().into_owned()))
      .unwrap_or_else(|| msg.author.name.clone());
    let url = format!("https://cdn.discordapp.com/emojis/{}.{}", blob.0, if blob.1 { "gif" } else { "png" });

    if let Err(e) = msg.delete() {
      warn!("could not delete message: {}", e);
    }

    Ok(CommandSuccess::default()
      .message(move |e: CreateEmbed| e
        .image(&url)
        .author(|a| a.name(&name))))
  }
}

lazy_static! {
  static ref BLOBS: HashMap<&'static str, (u64, bool)> = {
    let mut map = HashMap::new();
    map.insert("ablobaww", (396521774461747210, true));
    map.insert("ablobbounce", (399743288673959947, true));
    map.insert("ablobcheer", (399742793976643585, true));
    map.insert("ablobcouple", (396521402661994496, true));
    map.insert("ablobcry", (396521773065175043, true));
    map.insert("ablobdizzy", (399744982149496832, true));
    map.insert("ablobdrool", (396521401491783681, true));
    map.insert("ablobflushed", (397563618729656322, true));
    map.insert("ablobfrown", (396521732044750848, true));
    map.insert("ablobgift", (396514816891813888, true));
    map.insert("ablobgrimace", (396521772692013067, true));
    map.insert("ablobgrin", (396521732007264256, true));
    map.insert("ablobhearteyes", (396521773358776341, true));
    map.insert("ablobkiss", (396521773060849674, true));
    map.insert("abloblurk", (396521773581074432, true));
    map.insert("ablobmaracas", (397563059490521089, true));
    map.insert("ablobmelt", (396521773602045955, true));
    map.insert("ablobnervous", (396521731835035658, true));
    map.insert("ablobnom", (396521774147436554, true));
    map.insert("ablobnompopcorn", (396521773677412352, true));
    map.insert("ablobowo", (396521773660635137, true));
    map.insert("ablobpats", (396521773723549696, true));
    map.insert("ablobpeek", (396521773602177025, true));
    map.insert("ablobreach", (396521773266632715, true));
    map.insert("ablobreachreverse", (396521773727875072, true));
    map.insert("ablobsadcloud", (396521263247654912, true));
    map.insert("ablobsalute", (396521773484605440, true));
    map.insert("ablobshake", (396521263108980736, true));
    map.insert("ablobsleep", (396521731147169804, true));
    map.insert("ablobsmile", (396521732137025546, true));
    map.insert("ablobsunglasses", (399749059860103168, true));
    map.insert("ablobsweats", (396521773719617537, true));
    map.insert("ablobthinkingfast", (396521773622886403, true));
    map.insert("ablobtonguewink", (396521731415867392, true));
    map.insert("ablobunamused", (397525278626152448, true));
    map.insert("ablobuwu", (396521773702578177, true));
    map.insert("ablobwave", (396521774604615690, true));
    map.insert("ablobwavereverse", (396521774466203659, true));
    map.insert("ablobweary", (396521731340369921, true));
    map.insert("ablobwink", (396521731289776129, true));
    map.insert("acongablob", (396521772687818753, true));
    map.insert("ajakeblob", (396514816522452995, true));
    map.insert("apartyblob", (396521772758990851, true));
    map.insert("aphotoblob", (396521773694320640, true));
    map.insert("arainblob", (399748953156878346, true));
    map.insert("athinkingwithblobs", (396521772838682626, true));
    map.insert("b1nzyblob", (396521773115637810, false));
    map.insert("b4nzyblob", (396521772905922562, false));
    map.insert("blob0w0", (396521773652508684, false));
    map.insert("blobamused", (396521401403703297, false));
    map.insert("blobangel", (396521732086693888, false));
    map.insert("blobangery", (396521401466617858, false));
    map.insert("blobangry", (396521731432382474, false));
    map.insert("blobastonished", (396521731285712897, false));
    map.insert("blobawkward", (396521401672007681, false));
    map.insert("blobaww", (396521773023232000, false));
    map.insert("blobbandage", (396521401672269825, false));
    map.insert("blobblush", (396521772691881987, false));
    map.insert("blobbored", (396521401651298315, false));
    map.insert("blobbowing", (396521731298426891, false));
    map.insert("blobcheeky", (396521731516399626, false));
    map.insert("blobcheer", (396521773048266752, false));
    map.insert("blobconfounded", (396521401366085644, false));
    map.insert("blobconfused", (396521731520462858, false));
    map.insert("blobcool", (396521731373793281, false));
    map.insert("blobcouncil", (396521262614052904, false));
    map.insert("blobcouple", (396521401818808330, false));
    map.insert("blobcowboy", (396521401923665920, false));
    map.insert("blobcry", (396521731319267333, false));
    map.insert("blobdancer", (396521401789579274, false));
    map.insert("blobdead", (396521401810419712, false));
    map.insert("blobderpy", (396521401429000193, false));
    map.insert("blobdetective", (396521401843974155, false));
    map.insert("blobdevil", (396514815973261312, false));
    map.insert("blobdizzy", (396521731889692675, false));
    map.insert("blobdrool", (396521401638453249, false));
    map.insert("blobexpressionless", (396521401672269826, false));
    map.insert("blobeyesdown", (396521401844236288, false));
    map.insert("blobeyesup", (397149994555277314, false));
    map.insert("blobfacepalm", (396521401848168457, false));
    map.insert("blobfearful", (396521731247833091, false));
    map.insert("blobfistbumpL", (396521401907019776, false));
    map.insert("blobfistbumpR", (396521401856819200, false));
    map.insert("blobflushed", (396521401894436884, false));
    map.insert("blobfrown", (396521731537502208, false));
    map.insert("blobfrowning", (396521731537240074, false));
    map.insert("blobfrowningbig", (396521731529113600, false));
    map.insert("blobgift", (396514815855558679, false));
    map.insert("blobglare", (396521773102792724, false));
    map.insert("blobgo", (396521262417182721, false));
    map.insert("blobgrin", (396521732443471892, false));
    map.insert("blobhammer", (396521773149192215, false));
    map.insert("blobheart", (396521773119832064, false));
    map.insert("blobhearteyes", (396521731176529931, false));
    map.insert("blobhero", (396521401730990082, false));
    map.insert("blobhighfive", (396521262609858560, false));
    map.insert("blobhug", (396521731126460427, false));
    map.insert("blobhuh", (399743182054752257, false));
    map.insert("blobhyperthink", (396521772771442689, false));
    map.insert("blobhyperthinkfast", (396521401840041985, false));
    map.insert("blobhypesquad", (396521262706589706, false));
    map.insert("blobidea", (396521262643412992, false));
    map.insert("blobjoy", (396521731507879936, false));
    map.insert("blobkiss", (396521403224031233, false));
    map.insert("blobkissblush", (396521401537789953, false));
    map.insert("blobkissheart", (396521773115637780, false));
    map.insert("bloblul", (399742963112083461, false));
    map.insert("blobmelt", (396521772658458626, false));
    map.insert("blobmoustache", (396521402070728704, false));
    map.insert("blobnauseated", (396521401902694410, false));
    map.insert("blobnerd", (396521401982648320, false));
    map.insert("blobnervous", (396521731587833857, false));
    map.insert("blobneutral", (396521731319398401, false));
    map.insert("blobninja", (396514816082182144, false));
    map.insert("blobnogood", (396521731386376204, false));
    map.insert("blobnom", (396521773149192214, false));
    map.insert("blobnomchristmas", (396521262639480832, false));
    map.insert("blobnomcookie", (396521772855590916, false));
    map.insert("blobnomouth", (396521401932054538, false));
    map.insert("blobnostar", (396521262685356032, false));
    map.insert("blobnotsureif", (396521262501068812, false));
    map.insert("blobok", (396521731625582592, false));
    map.insert("blobokhand", (396521401860751362, false));
    map.insert("blobonfire", (396521401810550785, false));
    map.insert("blobopenmouth", (396521731575250944, false));
    map.insert("bloboro", (396514815973130240, false));
    map.insert("bloboutage", (396514815863947266, false));
    map.insert("blobowo", (396521773119832074, false));
    map.insert("blobowoevil", (396521773010649090, false));
    map.insert("blobparty", (397150027748868096, false));
    map.insert("blobpatrol", (396514815788449803, false));
    map.insert("blobpats", (396521773748846592, false));
    map.insert("blobpeek", (396521773245530124, false));
    map.insert("blobpensive", (396521731562668032, false));
    map.insert("blobpin", (396521262656258048, false));
    map.insert("blobpolice", (396521773048528900, false));
    map.insert("blobpoliceangry", (396521401986580480, false));
    map.insert("blobpopcorn", (396521772716916737, false));
    map.insert("blobpopsicle", (396521262706589707, false));
    map.insert("blobpout2", (396521401936511004, false));
    map.insert("blobpray", (396521731348496385, false));
    map.insert("blobreach", (396521773144866816, false));
    map.insert("blobreachdrool", (396521262710652938, false));
    map.insert("blobreachreverse", (396521773178552330, false));
    map.insert("blobrofl", (396521731617062912, false));
    map.insert("blobrollingeyes", (396521732258791424, false));
    map.insert("blobross", (396514816019267595, false));
    map.insert("blobsad", (396521773144866826, false));
    map.insert("blobsadcloud", (396521262714716160, false));
    map.insert("blobsadreach", (396521401789710347, false));
    map.insert("blobsalute", (396521773103054850, false));
    map.insert("blobsanta", (396514815742443521, false));
    map.insert("blobscream", (396521731600416778, false));
    map.insert("blobshh", (396521401831391233, false));
    map.insert("blobshrug", (396521773186809856, false));
    map.insert("blobsleeping", (396521731336175617, false));
    map.insert("blobsleepless", (396521402045300737, false));
    map.insert("blobsmile", (396521731440771085, false));
    map.insert("blobsmilehappy", (396521732267311104, false));
    map.insert("blobsmilehappyeyes", (396521731331850241, false));
    map.insert("blobsmileopenmouth", (396521731616931850, false));
    map.insert("blobsmileopenmouth2", (396521731264872453, false));
    map.insert("blobsmilesweat", (396521731637903370, false));
    map.insert("blobsmilesweat2", (396521731679977482, false));
    map.insert("blobsmiley", (396521731633709067, false));
    map.insert("blobsmirk", (396521731185180684, false));
    map.insert("blobsneezing", (396521401760088066, false));
    map.insert("blobsnuggle", (396521772738019339, false));
    map.insert("blobsob", (396521773115637783, false));
    map.insert("blobsplosion", (396521401852624908, false));
    map.insert("blobspy", (396521262702395392, false));
    map.insert("blobstop", (396521262534361115, false));
    map.insert("blobsurprised", (396521402682966016, false));
    map.insert("blobsweats", (396521772994002955, false));
    map.insert("blobteefs", (396521402003357696, false));
    map.insert("blobthinking", (396521773132283914, false));
    map.insert("blobthinkingcool", (396521401806225410, false));
    map.insert("blobthinkingdown", (396521773727744000, false));
    map.insert("blobthinkingeyes", (396521402003488768, false));
    map.insert("blobthinkingfast", (396521402028523520, false));
    map.insert("blobthinkingglare", (396521773149192192, false));
    map.insert("blobthinkingsmirk", (396521401605029891, false));
    map.insert("blobthonkang", (396521773132414987, false));
    map.insert("blobthumbsdown", (396521402041106432, false));
    map.insert("blobthumbsup", (396521773161775104, false));
    map.insert("blobthump", (396521401994969089, false));
    map.insert("blobtilt", (396521401974259712, false));
    map.insert("blobtired", (396521731629645834, false));
    map.insert("blobtongue", (396521731428319233, false));
    map.insert("blobtonguewink", (396521732233494528, false));
    map.insert("blobtriumph", (396521731621388288, false));
    map.insert("blobugh", (396521731684040704, false));
    map.insert("blobunamused", (396521731285712898, false));
    map.insert("blobunsure", (396521401852624907, false));
    map.insert("blobupset", (396521731466199051, false));
    map.insert("blobupsidedown", (396521731663200256, false));
    map.insert("blobuwu", (396521773098860576, false));
    map.insert("blobwaitwhat", (396521262790344734, false));
    map.insert("blobwave", (396521772968837121, false));
    map.insert("blobwavereverse", (396521773216301056, false));
    map.insert("blobweary", (396521731663069191, false));
    map.insert("blobwhistle", (396521401978191882, false));
    map.insert("blobwink", (396521731658874891, false));
    map.insert("blobwizard", (396521262500937731, false));
    map.insert("blobwoah", (396521401806356482, false));
    map.insert("blobxd", (396521731679977473, false));
    map.insert("blobyum", (396521731612999692, false));
    map.insert("blobzippermouth", (396521401894436885, false));
    map.insert("bolb", (396521773723680768, false));
    map.insert("doggoblob", (396514816065404928, false));
    map.insert("feelsblobman", (396521772868042754, false));
    map.insert("ferretblob", (396514816098828298, false));
    map.insert("gentleblob", (396521262735687680, false));
    map.insert("googlebee", (396514815989907458, false));
    map.insert("googleblueheart", (396514816631767041, false));
    map.insert("googlecake", (396514815885049857, false));
    map.insert("googlecat", (396514816157548544, false));
    map.insert("googlecatface", (396514815692242945, false));
    map.insert("googlecatheart", (396514816195559435, false));
    map.insert("googledog", (396514816140771328, false));
    map.insert("googlefire", (396514815889375233, false));
    map.insert("googleghost", (396514816052822022, false));
    map.insert("googlegun", (396514815935512587, false));
    map.insert("googlemuscleL", (396514815998164993, false));
    map.insert("googlemuscleR", (396514816073662475, false));
    map.insert("googlepenguin", (396514816128450560, false));
    map.insert("googlerabbit", (396514816174325780, false));
    map.insert("googleredheart", (396514815855558658, false));
    map.insert("googlesheep", (396514815671140354, false));
    map.insert("googlesnake", (396514816149159946, false));
    map.insert("googleturtle", (396514816107479053, false));
    map.insert("googlewhale", (396514815956484108, false));
    map.insert("GreenTick", (396521773245530123, false));
    map.insert("jakeblob", (396514816107347968, false));
    map.insert("kirbyblob", (396514815977193493, false));
    map.insert("nellyblob", (396514816115605524, false));
    map.insert("nikoblob", (396514815998427140, false));
    map.insert("notlikeblob", (396521773178552331, false));
    map.insert("photoblob", (396521773245530132, false));
    map.insert("pikablob", (396514816170262528, false));
    map.insert("pusheenblob", (396514816400818176, false));
    map.insert("rainblob", (396514816292028416, false));
    map.insert("RedTick", (396521773207912468, false));
    map.insert("reindeerblob", (396514816291897344, false));
    map.insert("rickblob", (396514816317194240, false));
    map.insert("thinkingwithblobs", (396521773325090817, false));
    map.insert("wolfiriblob", (396521772842745857, false));
    map.insert("wumpusblob", (396514816468189184, false));
    map
  };
}
