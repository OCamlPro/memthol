//! Tests.

use crate::*;

macro_rules! unwrap {
    ($e:expr) => {
        match $e {
            Ok(res) => res,
            Err(e) => {
                println!("{}", e.pretty());
                panic!("trying to unwrap an `Err` value")
            }
        }
    };
}

#[test]
fn position_details() {
    let parser = Parser::new(
        "\
first line with some text
second line problem is >H<ere
third line
    ",
    );

    let err = parser.position_details(50);

    let mut expected = "".to_string();
    expected.push_str("   |\n");
    expected.push_str(" 2 | second line problem is >H<ere\n");
    expected.push_str("   |                         ^");

    println!("{}", err);

    assert_eq! { err.to_string(), expected }
}

static DIFF_0: &str = r#"
0.039
new {
    212: Minor 182 [ `set.ml`:133:21-28#7 `set.ml`:130:21-28#2 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#26 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.039 _
    211: Minor 620 [ `set.ml`:130:21-28#2 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#27 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.039 _
    210: Minor 549 [ `set.ml`:133:21-28#1 `set.ml`:130:21-28#1 `set.ml`:133:21-28#7 `set.ml`:130:21-28#2 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#27 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.039 _
    209: Minor 223 [ `set.ml`:105:25-43#1 `set.ml`:133:21-28#6 `set.ml`:130:21-28#2 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#28 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.039 _
    208: Minor 269 [ `src/test.ml`:36:24-64#123 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.038 _
    207: Minor 520 [ `src/test.ml`:36:24-64#157 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.038 _
    206: Minor 277 [ `src/test.ml`:36:24-64#167 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.038 _
    205: Minor 185 [ `src/test.ml`:36:24-64#36 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.038 _
    204: Minor 166 [ `src/test.ml`:36:24-64#154 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.038 _
    203: Minor 431 [ `src/test.ml`:36:24-64#162 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.037 0.037
    202: Minor 489 [ `src/test.ml`:36:24-64#188 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.037 0.037
    201: Minor 554 [ `set.ml`:133:21-28#5 `set.ml`:130:21-28#1 `src/test.ml`:36:24-46#1 `src/test.ml`:36:24-64#30 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.037 0.037
    200: Minor 594 [ `src/test.ml`:36:24-64#97 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.037 0.037
    199: Minor 671 [ `src/test.ml`:36:24-64#36 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.037 0.037
    198: Minor 277 [ `src/test.ml`:36:24-64#49 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.036 0.037
    197: Minor 235 [ `src/test.ml`:36:24-64#146 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.036 0.037
    196: Minor 177 [ `src/test.ml`:36:24-64#232 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.036 0.037
    195: Minor 672 [ `src/test.ml`:36:24-64#233 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.036 0.037
    194: Minor 38 [ `src/test.ml`:36:24-64#53 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.036 0.037
    193: Minor 430 [ `src/test.ml`:36:24-64#123 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.035 0.037
    192: Minor 557 [ `src/test.ml`:36:24-64#128 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.035 0.035
    191: Minor 120 [ `src/test.ml`:36:24-64#43 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.035 0.035
    190: Minor 494 [ `src/test.ml`:36:24-64#163 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.035 0.035
    189: Minor 57 [ `src/test.ml`:36:24-64#133 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.035 0.035
    188: Minor 177 [ `src/test.ml`:36:24-64#124 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.034 0.035
    187: Minor 387 [ `src/test.ml`:36:24-64#124 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.034 0.035
    186: Minor 675 [ `src/test.ml`:36:24-64#132 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.034 0.035
    185: Minor 664 [ `set.ml`:133:21-28#1 `set.ml`:130:21-28#1 `set.ml`:133:21-28#7 `set.ml`:130:21-28#1 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#39 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.034 0.034
    184: Minor 168 [ `src/test.ml`:36:24-64#136 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.034 0.034
    183: Minor 661 [ `src/test.ml`:36:24-64#212 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.033 0.034
    182: Minor 467 [ `src/test.ml`:36:24-64#188 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.033 0.034
    181: Minor 68 [ `src/test.ml`:36:24-64#122 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.033 0.034
    180: Minor 542 [ `src/test.ml`:36:24-64#77 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.033 0.034
    179: Minor 383 [ `src/test.ml`:36:24-64#47 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.032 0.034
    178: Minor 441 [ `src/test.ml`:36:24-64#122 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.032 0.034
    177: Minor 681 [ `src/test.ml`:36:24-64#132 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.031 0.032
    176: Minor 216 [ `src/test.ml`:36:24-64#213 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.031 0.032
    175: Minor 400 [ `set.ml`:133:21-28#1 `set.ml`:130:21-28#1 `src/test.ml`:36:24-46#1 `src/test.ml`:36:24-64#43 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.031 0.032
    174: Minor 515 [ `set.ml`:133:21-28#1 `set.ml`:130:21-28#1 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#43 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.031 0.031
    173: Minor 596 [ `src/test.ml`:36:24-64#195 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.031 0.031
    172: Minor 319 [ `src/test.ml`:36:24-64#177 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.031 0.031
    171: Minor 156 [ `set.ml`:133:21-28#5 `set.ml`:130:21-28#1 `src/test.ml`:36:24-46#1 `src/test.ml`:36:24-64#44 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.030 0.031
    170: Minor 615 [ `src/test.ml`:36:24-64#184 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.030 0.031
    169: Minor 661 [ `src/test.ml`:36:24-64#207 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.030 0.031
    168: Minor 274 [ `src/test.ml`:36:24-64#211 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.030 0.031
    167: Minor 3 [ `src/test.ml`:36:24-64#100 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.029 0.030
    166: Minor 583 [ `src/test.ml`:36:24-64#195 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.029 0.030
    165: Minor 288 [ `src/test.ml`:36:24-64#158 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.029 0.030
    164: Minor 300 [ `set.ml`:133:21-28#6 `set.ml`:130:21-28#2 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#51 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.028 0.030
    163: Minor 599 [ `src/test.ml`:36:24-64#99 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.028 0.030
    162: Minor 252 [ `src/test.ml`:36:24-64#169 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.028 0.030
    161: Minor 275 [ `src/test.ml`:36:24-64#164 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.028 0.028
    160: Minor 541 [ `src/test.ml`:36:24-64#181 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.028 0.028
    159: Minor 171 [ `src/test.ml`:36:24-64#181 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.027 0.028
    158: Minor 238 [ `src/test.ml`:36:24-64#196 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.027 0.028
    157: Minor 78 [ `src/test.ml`:36:24-64#129 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.027 0.028
    156: Minor 616 [ `src/test.ml`:36:24-64#105 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.027 0.028
    155: Minor 259 [ `set.ml`:130:21-28#1 `set.ml`:133:21-28#6 `set.ml`:130:21-28#2 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#56 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.026 0.027
    154: Minor 433 [ `src/test.ml`:36:24-64#194 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.026 0.027
    153: Minor 157 [ `src/test.ml`:36:24-64#171 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.026 0.027
    152: Minor 392 [ `src/test.ml`:36:24-64#177 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.026 0.027
    151: Minor 43 [ `src/test.ml`:36:24-64#161 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.025 0.027
    150: Minor 48 [ `src/test.ml`:36:24-64#112 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.025 0.027
    149: Minor 238 [ `src/test.ml`:36:24-64#141 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.025 0.025
    148: Minor 691 [ `src/test.ml`:36:24-64#147 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.025 0.025
    147: Minor 487 [ `src/test.ml`:36:24-64#179 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.024 0.025
    146: Minor 681 [ `src/test.ml`:36:24-46#1 `src/test.ml`:36:24-64#59 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.024 0.024
    145: Minor 23 [ `set.ml`:133:21-28#2 `set.ml`:130:21-28#1 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#61 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.024 0.024
    144: Minor 527 [ `set.ml`:133:21-28#5 `set.ml`:130:21-28#2 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#63 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.024 0.024
    143: Minor 476 [ `set.ml`:130:21-28#1 `set.ml`:133:21-28#6 `set.ml`:130:21-28#2 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#63 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.024 0.024
    142: Minor 680 [ `src/test.ml`:36:24-64#122 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.024 0.024
    141: Minor 482 [ `src/test.ml`:36:24-64#127 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.023 0.024
    140: Minor 371 [ `src/test.ml`:36:24-64#170 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.023 0.024
    139: Minor 549 [ `src/test.ml`:36:24-64#160 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.023 0.024
    138: Minor 316 [ `src/test.ml`:36:24-64#181 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.022 0.022
    137: Minor 610 [ `src/test.ml`:36:24-64#76 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.022 0.022
    136: Minor 365 [ `src/test.ml`:36:24-64#161 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.022 0.022
    135: Minor 517 [ `src/test.ml`:36:24-64#118 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.022 0.022
    134: Minor 603 [ `src/test.ml`:36:24-64#79 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.021 0.022
    133: Minor 632 [ `src/test.ml`:36:24-64#127 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.021 0.022
    132: Minor 393 [ `src/test.ml`:36:24-64#155 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.021 0.022
    131: Minor 3 [ `src/test.ml`:36:24-64#179 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.021 0.021
    130: Minor 446 [ `src/test.ml`:36:24-64#174 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.021 0.021
    129: Minor 211 [ `src/test.ml`:36:24-64#92 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.020 0.021
    128: Minor 665 [ `src/test.ml`:36:24-64#105 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.020 0.021
    127: Minor 535 [ `src/test.ml`:36:24-64#93 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.020 0.021
    126: Minor 423 [ `src/test.ml`:36:24-64#137 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.020 0.021
    125: Minor 594 [ `src/test.ml`:36:24-64#165 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.020 0.021
    124: Minor 219 [ `src/test.ml`:36:24-64#131 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.019 0.019
    123: Minor 58 [ `src/test.ml`:36:24-64#101 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.019 0.019
    122: Minor 466 [ `src/test.ml`:36:24-64#114 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.019 0.019
    121: Minor 623 [ `src/test.ml`:36:24-64#162 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.019 0.019
    120: Minor 127 [ `src/test.ml`:36:24-64#92 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.018 0.019
    119: Minor 88 [ `src/test.ml`:36:24-64#93 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.018 0.019
    118: Minor 614 [ `src/test.ml`:36:24-64#137 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.018 0.019
    117: Minor 295 [ `src/test.ml`:36:24-64#116 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.018 0.019
    116: Minor 377 [ `src/test.ml`:36:24-64#140 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.017 0.018
    115: Minor 561 [ `src/test.ml`:36:24-64#107 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.017 0.018
    114: Minor 655 [ `src/test.ml`:36:24-64#94 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.017 0.018
    113: Minor 615 [ `src/test.ml`:36:24-64#112 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.017 0.018
    112: Minor 358 [ `src/test.ml`:36:24-64#168 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.017 0.017
    111: Minor 78 [ `src/test.ml`:36:24-64#119 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.016 0.017
    110: Minor 205 [ `src/test.ml`:36:24-64#99 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.016 0.017
    109: Minor 9 [ `src/test.ml`:36:24-64#154 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.016 0.017
    108: Minor 679 [ `src/test.ml`:36:24-64#112 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.016 0.017
    107: Minor 29 [ `src/test.ml`:36:24-64#105 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.016 0.017
    106: Minor 619 [ `src/test.ml`:36:24-64#164 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.015 0.017
    105: Minor 24 [ `src/test.ml`:36:24-64#113 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.015 0.015
    104: Minor 582 [ `src/test.ml`:36:24-64#109 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.015 0.015
    103: Minor 82 [ `src/test.ml`:36:24-64#103 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.015 0.015
    102: Minor 533 [ `src/test.ml`:36:24-64#138 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.014 0.015
    101: Minor 461 [ `src/test.ml`:36:24-64#154 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.014 0.015
    100: Minor 359 [ `src/test.ml`:36:24-64#156 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.014 0.015
    99: Minor 677 [ `src/test.ml`:36:24-64#116 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.014 0.015
    98: Minor 396 [ `src/test.ml`:36:24-64#125 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.013 0.013
    97: Minor 429 [ `src/test.ml`:36:24-64#114 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.013 0.013
    96: Minor 145 [ `src/test.ml`:36:24-64#149 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.013 0.013
    95: Minor 283 [ `src/test.ml`:36:24-64#118 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.013 0.013
    94: Minor 379 [ `src/test.ml`:36:24-64#116 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.012 0.013
    93: Minor 635 [ `src/test.ml`:36:24-64#154 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.012 0.013
    92: Minor 177 [ `src/test.ml`:36:24-64#118 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.012 0.013
    91: Minor 257 [ `src/test.ml`:36:24-64#134 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.012 0.012
    90: Minor 206 [ `src/test.ml`:36:24-64#134 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.011 0.012
    89: Minor 626 [ `src/test.ml`:36:24-64#80 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.011 0.012
    88: Minor 132 [ `src/test.ml`:36:24-46#1 `src/test.ml`:36:24-64#60 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.011 0.012
    87: Minor 223 [ `src/test.ml`:36:24-46#1 `src/test.ml`:36:24-64#59 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.011 0.012
    86: Minor 9 [ `set.ml`:130:21-28#7 `set.ml`:133:21-28#2 `src/test.ml`:36:24-46#1 `src/test.ml`:36:24-64#43 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.011 0.012
    85: Minor 681 [ `set.ml`:130:21-28#1 `set.ml`:133:21-28#2 `src/test.ml`:36:24-46#1 `src/test.ml`:36:24-64#37 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.011 0.012
    84: Minor 57 [ `src/test.ml`:36:24-46#1 `src/test.ml`:36:24-64#19 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.011 0.012
    83: Minor 533 [ `set.ml`:130:21-28#1 `set.ml`:133:21-28#3 `set.ml`:130:21-28#1 `set.ml`:133:21-28#2 `src/test.ml`:36:24-46#1 `src/test.ml`:36:24-64#14 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.011 0.012
    82: Minor 93 [ `set.ml`:105:48-64#1 `set.ml`:133:21-28#5 `set.ml`:130:21-28#2 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#11 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.011 0.012
    81: Minor 474 [ `src/test.ml`:36:24-64#42 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.011 0.012
    80: Minor 253 [ `src/test.ml`:36:24-64#45 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.011 0.012
    79: Minor 687 [ `src/test.ml`:36:24-64#82 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.011 0.012
    78: Minor 290 [ `src/test.ml`:36:24-64#26 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.010 0.010
    77: Minor 667 [ `src/test.ml`:36:24-64#113 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.010 0.010
    76: Minor 33 [ `src/test.ml`:36:24-64#29 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.010 0.010
    75: Minor 288 [ `src/test.ml`:36:24-64#94 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.010 0.010
    74: Minor 497 [ `src/test.ml`:36:24-64#55 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.010 0.010
    73: Minor 474 [ `set.ml`:130:21-28#1 `set.ml`:133:21-28#7 `set.ml`:130:21-28#1 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#13 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.010 0.010
    72: Minor 629 [ `set.ml`:133:21-28#6 `set.ml`:130:21-28#2 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#15 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.010 0.010
    71: Minor 477 [ `src/test.ml`:36:24-64#114 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.010 0.010
    70: Minor 329 [ `src/test.ml`:36:24-64#26 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.009 0.010
    69: Minor 326 [ `set.ml`:130:21-28#2 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#18 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.009 0.010
    68: Minor 506 [ `src/test.ml`:36:24-64#68 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.009 0.010
    67: Minor 116 [ `set.ml`:130:21-28#2 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#21 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.009 0.010
    66: Minor 624 [ `src/test.ml`:36:24-64#56 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.009 0.010
    65: Minor 99 [ `src/test.ml`:36:24-64#117 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.009 0.010
    64: Minor 285 [ `src/test.ml`:36:24-64#34 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.009 0.010
    63: Minor 644 [ `src/test.ml`:36:24-64#46 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.009 0.010
    62: Minor 70 [ `src/test.ml`:36:24-64#92 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.008 0.010
    61: Minor 119 [ `src/test.ml`:36:24-64#37 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.008 0.010
    60: Minor 118 [ `set.ml`:133:21-28#1 `set.ml`:130:21-28#1 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#32 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.008 0.010
    59: Minor 531 [ `src/test.ml`:36:24-64#40 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.008 0.010
    58: Minor 264 [ `src/test.ml`:36:24-64#36 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.008 0.010
    57: Minor 242 [ `set.ml`:112:21-36#1 `set.ml`:133:21-28#4 `set.ml`:130:21-28#2 `src/test.ml`:38:20-42#1 `src/test.ml`:36:24-64#36 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.008 0.010
    56: Minor 191 [ `src/test.ml`:36:24-64#81 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.008 0.010
    55: Minor 421 [ `src/test.ml`:36:24-64#102 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.008 0.008
    54: Minor 475 [ `src/test.ml`:36:24-64#50 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.008 0.008
    53: Minor 659 [ `src/test.ml`:36:24-64#52 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.007 0.008
    52: Minor 589 [ `set.ml`:133:21-28#4 `set.ml`:130:21-28#1 `src/test.ml`:36:24-46#1 `src/test.ml`:36:24-64#40 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.007 0.008
    51: Minor 62 [ `src/test.ml`:36:24-64#88 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.007 0.008
    50: Minor 476 [ `src/test.ml`:36:24-64#52 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.007 0.008
    49: Minor 488 [ `src/test.ml`:36:24-64#64 `src/test.ml`:48:21-28#1 `src/test.ml`:55:12-21#2 `src/test.ml`:77:12-30#1 `src/memthol.ml`:96:22-26#1 `src/test.ml`:71:8-240#1 ] 0.007 0.008
}
dead {
    48: 0.007
    47: 0.007
    46: 0.007
    45: 0.007
    44: 0.007
    43: 0.007
    42: 0.007
    41: 0.007
    40: 0.007
    39: 0.007
    38: 0.007
    37: 0.007
    36: 0.007
    35: 0.007
    34: 0.007
    33: 0.007
    32: 0.007
    31: 0.007
    30: 0.007
    29: 0.007
    28: 0.007
}
"#;

#[test]
fn diff_0() {
    let mut parser = Parser::new(DIFF_0);
    let diff = unwrap!(parser.diff());
    assert_eq! { diff.new.len(), 164 }
    assert_eq! { diff.dead.len(), 21 }
}
