use chrono::{DateTime, Local, TimeZone};

// 一人暮らしの契約、引越し
const INITIAL_LIVING_ALONE_EXPENSE: u32 = 480000;

// 一人暮らしの家賃、仕送り、更新料の年割
const ANNUAL_LIVING_ALONE_EXPENSE: u32 = 200000 * 12 + 40000;

// 寿命
const LIFESPAN_YEARS: u8 = 80;

// 結婚年齢
const MARRIAGE_AGE: u8 = 30;

// 親との年齢差
const DIFF_FROM_PARENT_AGE: u8 = 35;

// 免許取得年齢
const DRIVER_LICENCE_AQUISITION_AGE: u8 = 18;

// 下の URL の調査における 2022 年東京都区部の世帯人数
// https://www.stat.go.jp/data/kakei/2022np/index.html
const GOV_STAT_FAMILIY_NUM_PEOPLE: f64 = 2.87;

// 東京の単身世帯の食費から世帯の食費の一人分とベース分を分割
// https://www.stat.go.jp/data/kakei/2022np/index.html
const PERSON_FOOD_EXPENSE: u32 = (((87973.0 - 39069.0) / (GOV_STAT_FAMILIY_NUM_PEOPLE - 1.0)) * 12.0) as u32;
const BASE_FOOD_EXPENSE: u32 = 39069 * 12 - PERSON_FOOD_EXPENSE;

// 子供の結婚への支援
// https://souken.zexy.net/research_news/trend.html
const CHILD_MARRIAGE_SUPPORT_EXPENSE: u32 = 1932000;

// 葬式にかかる費用
// https://prtimes.jp/main/html/rd/p/000000019.000020574.html
const PARENT_FUNERAL_EXPENSE: u32 = 1861000;

// 光熱、電気、ガス、水道（世帯の値から単身世帯を引いて、一人当たりとベースに分割）
// https://www.stat.go.jp/data/kakei/2022np/index.html
const PERSON_FUEL_LIGHT_WATER_GAS_ETC_EXPENSE: u32 = (((22846.0 - 13098.0) / (GOV_STAT_FAMILIY_NUM_PEOPLE - 1.0)) * 12.0) as u32;
const BASE_FUEL_LIGHT_WATER_GAS_ETC_EXPENSE: u32 = 13098 * 12 - PERSON_FUEL_LIGHT_WATER_GAS_ETC_EXPENSE;

// 家具
// https://www.stat.go.jp/data/kakei/2022np/index.html
const PERSON_FURNITURE_EXPENSE: u32 = (((11587.0 - 5487.0) / (GOV_STAT_FAMILIY_NUM_PEOPLE - 1.0)) * 12.0) as u32;
const BASE_FURNITURE_EXPENSE: u32 = 5487 * 12 - PERSON_FURNITURE_EXPENSE;

// 衣類
// 世帯の衣類の支出から一人あたりと、単身世帯だと単身世帯の方が少し（20%ほど）高いが、そちらをベースにする
// https://www.stat.go.jp/data/kakei/2022np/index.html
const CLOTHING_EXPENSE: u32 = 5047 * 12;

// 東京の教習所の平均額
// https://hajimen.com/12-tokyo/rank_detail
const DRIVER_LICENCE_AQUISITION_EXPENSE: u32 = 302489;

// 携帯の月額料金と買い替え料金を均した額の予測値
const MOBILE_EXPENSE: u32 = 3000 * 12 + 10000;

#[derive(Debug)]
pub struct Person {
    pub name: String,

    pub birth_date: DateTime<Local>,

    // 子供の場合 true, 配偶者や自分は false
    pub is_child: bool,
}

impl Person {

    // 学年を決める4月2日時点での年齢
    // 生まれてない or 寿命を過ぎてたら None
    pub fn get_grade_age(&self, year: u16) -> Option<u8> {
        let tz = self.birth_date.timezone();
        let base_date = tz.with_ymd_and_hms(year.into(), 4, 2, 0, 0, 0).unwrap();
        match base_date.years_since(self.birth_date) {
            None => None,
            Some(years) => if years > LIFESPAN_YEARS.into() {
                None
            } else {
                Some(years as u8)
            }
        }
    }
}

pub struct Car {
    pub start_year: u16, // include
    pub end_year: u16, // not include
    pub annual_car_type_tax: u32, // 自動車（種別割）税の年割
    pub annual_weight_tax: u32, // 自動車重量税
    pub annual_liability_insurance_fee: u32, // 自賠責の年割
    pub annual_optional_insurance_fee: u32, // 任意保険の年割
    pub annual_inspection_fee: u32, // 車検代の年割
    pub annual_gas_expense: u32, // 年間のガソリン代
    pub annual_consumables_expense: u32, // 年間の消耗品
    pub down_payment: u32,
    pub loan: Option<YearlyLoan>,
}

impl Car {
    pub fn estimate_expense(&self, year: u16) -> u32 {
        let mut expense = 0;
        if self.start_year <= year && year < self.end_year {
            // いわゆる維持費
            expense += self.annual_car_type_tax + 
                self.annual_weight_tax + 
                self.annual_liability_insurance_fee + 
                self.annual_optional_insurance_fee + 
                self.annual_inspection_fee + 
                self.annual_gas_expense + 
                self.annual_consumables_expense;
        };
        if self.start_year == year {
            // 初期費
            expense += self.down_payment;
        };
        if let Some(loan) = &self.loan {
            // ローンの支払い
            if self.start_year <= year && year < (self.start_year + loan.payment_years) {
                expense += loan.calcurate_yearly_payment();
            }
        };
        expense
    }
}

pub struct House {
    pub start_year: u16, // include
    pub end_year: u16, // not include
    pub moving_expense: u32,
    pub kind: HouseKind,
}

pub enum HouseKind {
    Rental {
        rent: u32,
    },
    Own {
        down_payment: u32,
        loan: Option<YearlyLoan>,
    },
}

impl House {
    pub fn estimate_expense(&self, year: u16) -> u32 {
        let mut expense = 0;
        if self.start_year == year {
            expense += self.moving_expense;
        }
        match &self.kind {
            HouseKind::Rental { rent } => {
                if self.start_year <= year && year < self.end_year {
                    if self.start_year == year {
                        expense += rent * 2; // 敷金礼金
                    } else {
                        let residence_years = year - self.start_year;
                        if residence_years % 2 == 0 {
                            expense += rent; // 契約更新料
                        };
                    };
                    expense += rent * 12; // 家賃
                };
            },
            HouseKind::Own { down_payment, loan } => {
                if self.start_year == year {
                    // 初期費
                    expense += down_payment;
                };
                if let Some(loan) = loan {
                    if self.start_year <= year && year < (self.start_year + loan.payment_years) {
                        expense += loan.calcurate_yearly_payment();
                    }
                };
            },
        }
        expense
    }
}

pub struct YearlyLoan {
    pub interest_rate: f64,
    pub payment_years: u16,
    pub amount: u32,
}

impl YearlyLoan {
    fn calcurate_monthly_payment(&self) -> u32 {
        let interest_rate = self.interest_rate / 12.0;
        let months = self.payment_years * 12;
        let pvif = (interest_rate + 1.0).powf(months as f64);
        let payment = interest_rate / (pvif - 1.0) * -(self.amount as f64 * pvif);
        (-payment) as u32
    }

    pub fn calcurate_yearly_payment(&self) -> u32 {
        self.calcurate_monthly_payment() * 12
    }
}

#[derive(PartialEq, Eq)]
pub enum LifeStage {
    PreSchool,
    KinderGarden,
    ElementarySchool,
    MiddleSchool,
    HighSchool,
    UnderGraduate,
    Masters,
    Doctorate,
    Working,
}

impl LifeStage {
    // 念の為、全て高めに見積もる (2023 年現在で集められる情報を元にしている)

    // 学費関連
    // https://www.metro.tokyo.lg.jp/tosei/hodohappyo/press/2022/12/07/07.html
    // https://www.seikatubunka.metro.tokyo.lg.jp/shigaku/sonota/files/0000000077/05shigaku_gyosei_4syo_R5.pdf
    // https://eic.obunsha.co.jp/pdf/educational_info/2022/0822_1.pdf
    // https://www.mext.go.jp/a_menu/koutou/shinkou/07021403/1412031_00004.htm

    pub fn new(age: u8)  -> LifeStage {
        match age {
            0..=2 => Self::PreSchool,
            3..=5 => Self::KinderGarden,
            6..=11 => Self::ElementarySchool,
            12..=14 => Self::MiddleSchool,
            15..=17 => Self::HighSchool,
            18..=21 => Self::UnderGraduate,
            22..=23 => Self::Masters,
            24..=25 => Self::Doctorate,
            _ => Self::Working,
        }
    }

    pub fn estimate_annual_tuition(&self) -> u32 {
        match self {
            Self::PreSchool => 0,
            Self::KinderGarden => 377077 + 17133 + 31962,
            Self::ElementarySchool => 552581 + 200522,
            Self::MiddleSchool => 492209 + 199759,
            Self::HighSchool => 483311 + 184399,
            Self::UnderGraduate => 967288,
            Self::Masters => 776040,
            Self::Doctorate => 628729,
            Self::Working => 0,
        }
    }

    pub fn estimate_initial_school_fees(&self) -> u32 {
        match self {
            Self::PreSchool => 0,
            Self::KinderGarden => 109166 + 5483,
            Self::ElementarySchool => 255357 + 52143 + 24446 * 2,
            Self::MiddleSchool => 263020 + 34137 + 23897 * 2,
            Self::HighSchool => 253113 + 39096 + 23322 * 3,
            Self::UnderGraduate => 1643466 - 967288 + 261004 * 3,
            Self::Masters => 76206 + 202598 * 3,
            Self::Doctorate => 51842 + 189623 * 3,
            Self::Working => 0,
        }
    }

    pub fn might_need_support_living_alone(&self) -> bool {
        match self {
            Self::PreSchool => false,
            Self::KinderGarden => false,
            Self::ElementarySchool => false,
            Self::MiddleSchool => false,
            Self::HighSchool => false,
            Self::UnderGraduate => true,
            Self::Masters => true,
            Self::Doctorate => true,
            Self::Working => false,
        }
    }
}

// 衣類
// 以下のスプレッドシートの計算により、年齢ごとの比率を決めた
// https://docs.google.com/spreadsheets/d/1O-reA7is_DVPTW-k1EU4e9Hc5f5Q6bPBGPWfEgM3Z_I/edit?usp=sharing
pub fn estimate_clothing_expense(age: u8, is_child: bool) -> u32 {
    if is_child {
        let stage = LifeStage::new(age);
        // 仕送りを想定している場合はそちらに含まれるので 0 を返す
        if stage.might_need_support_living_alone() || stage == LifeStage::Working {
            return 0;
        }
    }
    let rate = match age {
        0 => 0.904479703,
        1 => 0.7528228185,
        2 => 0.7174627804,
        3 => 0.6809886838,
        4 => 0.7081517868,
        5 => 0.61088451,
        6 => 0.6791319195,
        7 => 0.6869846738,
        8 => 0.7422166185,
        9 => 0.7433940788,
        10 => 0.8035622993,
        11 => 0.8303631068,
        12 => 0.9398669132,
        13 => 0.8913555495,
        14 => 0.7917605243,
        15..=17 => 1.108464734,
        18..=21 => 1.425168944,
        22..=23 => 1.741873153,
        24..=25 => 1.900225258,
        26..=29 => 2.058577363,
        30..=34 => 1.583521049,
        35..=39 => 1.266816839,
        40..=49 => 0.9501126292,
        50..=59 => 0.7917605243,
        _ => 0.6334084194,
    };
    (CLOTHING_EXPENSE as f64 * rate) as u32
}

// 食費
// 以下のスプレッドシートの計算により、年齢ごとの比率を決めた
// https://docs.google.com/spreadsheets/d/1O-reA7is_DVPTW-k1EU4e9Hc5f5Q6bPBGPWfEgM3Z_I/edit?usp=sharing
pub fn estimate_person_food_expense(age: u8, is_child: bool) -> u32 {
    if is_child {
        let stage = LifeStage::new(age);
        // 仕送りを想定している場合はそちらに含まれるので 0 を返す
        if stage.might_need_support_living_alone() || stage == LifeStage::Working {
            return 0;
        }
    }
    let rate = match age {
        0 => 0.3461706859,
        1 => 0.4840146905,
        2 => 0.6132080103,
        3 => 0.6628411973,
        4 => 0.7058329511,
        5 => 0.7106333491,
        6 => 0.7893866661,
        7 => 0.821466159,
        8 => 0.8453341988,
        9 => 0.8866998607,
        10 => 0.9566342765,
        11 => 0.9319469605,
        12 => 1.050059803,
        13 => 1.133703856,
        14 => 1.151011456,
        15..=17 => 1.174031685,
        18..=29 => 1.070440654,
        30..=49 => 1.093460883,
        50..=64 => 1.047420425,
        65..=74 => 0.9783597377,
        _ => 0.8632585921,
    };
    (PERSON_FOOD_EXPENSE as f64 * rate) as u32
}

// 医療費
// https://docs.google.com/spreadsheets/d/1O-reA7is_DVPTW-k1EU4e9Hc5f5Q6bPBGPWfEgM3Z_I/edit#gid=227018819
pub fn estimate_medical_expense(age: u8, is_child: bool) -> u32 {
    if is_child {
        let stage = LifeStage::new(age);

        // 仕送りを想定している場合はそちらに含む
        if stage.might_need_support_living_alone() || stage == LifeStage::Working {
            return 0;
        }
    }
    match age {
        0 => 15027,
        1 => 16168,
        2 => 12232,
        3 => 13030,
        4 => 14814,
        5 => 14209,
        6 => 20840,
        7 => 22906,
        8 => 26489,
        9 => 27330,
        10 => 23284,
        11 => 23256,
        12 => 23608,
        13 => 29707,
        14 => 21903,
        15..=19 => 19878,
        20..=24 => 19923,
        25..=29 => 24676,
        30..=34 => 28861,
        35..=39 => 32068,
        40..=44 => 36435,
        45..=49 => 44213,
        50..=54 => 56040,
        55..=59 => 88814,
        60..=64 => 88268,
        65..=69 => 110511,
        70..=74 => 93249,
        _ => 57867,
    }
}

// 学校外教育費（塾、予備校）
pub fn estimate_extra_education_expense(age: u8) -> u32 {
    match age {
        0 => 26809,
        1 => 34193,
        2 => 35877,
        3 => 44351,
        4 => 55861,
        5 => 55048,
        6 => 87941,
        7 => 91968,
        8 => 109036,
        9 => 129767,
        10 => 182600,
        11 => 224482,
        12 => 202826,
        13 => 251784,
        14 => 387870,
        15 => 400000,
        16 => 500000,
        17 => 1000000,
        _ => 0,
    }
}

// 習い事
pub fn estimate_extracurricular_activities_expense(age: u8) -> u32 {
    match age {
        0 => 28090,
        1 => 38108,
        2 => 45284,
        3 => 49827,
        4 => 77783,
        5 => 90609,
        6 => 100251,
        7 => 112791,
        8 => 117021,
        9 => 120734,
        10 => 113773,
        11 => 102553,
        12 => 88245,
        13 => 74513,
        14 => 72897,
        15 => 60000,
        16 => 50000,
        17 => 40000,
        _ => 0,
    }
}

// お小遣い、プレゼント（クリスマス、誕生日、ご褒美等）
pub fn estimate_allowance(age: u8, is_child: bool) -> u32 {
    if is_child {
        let stage = LifeStage::new(age);

        if stage.might_need_support_living_alone() {
            return 30000;
        }
        if stage == LifeStage::Working {
            return 20000;
        }
    };
    match age {
        0 => 51284,
        1 => 33325,
        2 => 29098,
        3 => 33985,
        4 => 30977,
        5 => 31913,
        6 => 43203,
        7 => 35816,
        8 => 39770,
        9 => 41291,
        10 => 42502,
        11 => 51609,
        12 => 61493,
        13 => 76845,
        14 => 80005,
        15 => 85000,
        16 => 90000,
        17 => 80000,
        _ => 20000,
    }
}

// 冠婚葬祭
pub fn estimate_ceremony_expense(age: u8, is_child: bool) -> u32 {
    let mut expense = 0;

    // 子供の結婚
    if is_child && age == MARRIAGE_AGE {
        expense += CHILD_MARRIAGE_SUPPORT_EXPENSE;
    }

    // 両親の葬式
    if !is_child && age == (LIFESPAN_YEARS - DIFF_FROM_PARENT_AGE) {
        expense += PARENT_FUNERAL_EXPENSE * 2;
    }

    expense
}

// レジャー、旅行
pub fn estimate_leisure_expense(age: u8) -> u32 {
    match age {
        0 => 79163,
        1 => 108488,
        2 => 125141,
        3 => 125299,
        4 => 141137,
        5 => 146071,
        6 => 152708,
        7 => 168690,
        8 => 182467,
        9 => 171124,
        10 => 177438,
        11 => 177201,
        12 => 173200,
        13 => 174405,
        14 => 123861,
        15 => 175000,
        16 => 175000,
        17 => 120000,
        _ => 175000,
    }
}

// 車の免許取得
pub fn estimate_driver_lincense_aquisition_fees(age: u8) -> u32 {
    if age == DRIVER_LICENCE_AQUISITION_AGE {
        DRIVER_LICENCE_AQUISITION_EXPENSE
    } else {
        0
    }
}

#[derive(Debug)]
pub struct FamilyExpense {
    pub car_expense: u32,
    pub house_expense: u32,
    pub food_expense: u32,
    pub fuel_light_water_gas_etc_expense: u32,
    pub furniture_expense: u32,
    pub member_expenses: Vec<PersonExpense>,
}

#[derive(Debug)]
pub struct PersonExpense {
    pub name: String,
    pub clothing_expense: u32,
    pub food_expense: u32,
    pub fuel_light_water_gas_etc_expense: u32,
    pub furniture_expense: u32,
    pub medical_expense: u32,
    pub education_expense: u32,
    pub extra_education_expense: u32,
    pub extracurricular_activities_expense: u32,
    pub mobile_expense: u32,
    pub allowance: u32,
    pub living_alone_expense: u32,
    pub ceremony_expense: u32,
    pub leisure_expense: u32,
    pub driver_lincense_aquisition_fees: u32,
}

pub fn estimate_family_expenses(people: Vec<Person>, cars: Vec<Car>, houses: Vec<House>, start_year: u16, years: u8) -> Vec<FamilyExpense> {
    let mut expenses = Vec::new();

    for year in start_year..(start_year + years as u16) {
        let car_expense = cars.iter().map(|car| car.estimate_expense(year)).fold(0, |sum, e| sum + e);
        let house_expense = houses.iter().map(|house| house.estimate_expense(year)).fold(0, |sum, e| sum + e);

        // per family expense
        let base_food_expense = BASE_FOOD_EXPENSE;
        let base_fuel_light_water_gas_etc_expense = BASE_FUEL_LIGHT_WATER_GAS_ETC_EXPENSE;
        let base_furniture_expense = BASE_FURNITURE_EXPENSE;

        let mut member_expenses = Vec::new();

        for person in &people {
            let Some(age) = person.get_grade_age(year) else {
                continue;
            };

            // 学校など現在の状態
            // 一人暮らしの状態
            let stage = LifeStage::new(age);
            let needs_living_alone_expense = stage.might_need_support_living_alone();
            let (needs_school_initial_fees, needs_initial_living_alone_expense) = if let Some(prev_age) = age.checked_sub(1) {
                let prev_stage = LifeStage::new(prev_age);
                (
                    person.is_child && stage != prev_stage,
                    person.is_child && !prev_stage.might_need_support_living_alone() && needs_living_alone_expense,
                )
            } else {
                (false, false)
            };

            // 衣類
            let clothing_expense = estimate_clothing_expense(age, person.is_child);

            // 食費
            let food_expense = estimate_person_food_expense(age, person.is_child);

            // 一人当たり光熱、ガス、水道、電気など
            let fuel_light_water_gas_etc_expense = if !needs_living_alone_expense && stage != LifeStage::Working {
                PERSON_FUEL_LIGHT_WATER_GAS_ETC_EXPENSE
            } else {
                0
            };

            // 一人当たり家具
            let furniture_expense = if !needs_living_alone_expense && stage != LifeStage::Working {
                PERSON_FURNITURE_EXPENSE
            } else {
                0
            };

            // 医療費
            let medical_expense = estimate_medical_expense(age, person.is_child);

            // 保育費、学費
            let mut education_expense = if needs_school_initial_fees {
                stage.estimate_initial_school_fees()
            } else {
                0
            };
            education_expense += stage.estimate_annual_tuition();

            // 学校外教育費（塾、予備校）
            let extra_education_expense = estimate_extra_education_expense(age);

            // 習い事
            let extracurricular_activities_expense = estimate_extracurricular_activities_expense(age);

            // 携帯電話（10歳から持つものとする）
            let mobile_expense = if 10 <= age {
                MOBILE_EXPENSE
            } else {
                0
            };

            // お小遣い、プレゼント（クリスマス、誕生日、ご褒美等）
            let allowance = estimate_allowance(age, person.is_child);

            // 一人暮らし開始
            let mut living_alone_expense = if needs_initial_living_alone_expense {
                INITIAL_LIVING_ALONE_EXPENSE
            } else {
                0
            };

            // 仕送り
            if needs_living_alone_expense {
                living_alone_expense += ANNUAL_LIVING_ALONE_EXPENSE;
            }

            // 冠婚葬祭
            let ceremony_expense = estimate_ceremony_expense(age, person.is_child);

            // レジャー、旅行
            let leisure_expense = estimate_leisure_expense(age);

            // 車の免許取得
            let driver_lincense_aquisition_fees = estimate_driver_lincense_aquisition_fees(age);

            member_expenses.push(PersonExpense {
                name: person.name.clone(),
                clothing_expense,
                food_expense,
                fuel_light_water_gas_etc_expense,
                furniture_expense,
                medical_expense,
                education_expense,
                extra_education_expense,
                extracurricular_activities_expense,
                mobile_expense,
                allowance,
                living_alone_expense,
                ceremony_expense,
                leisure_expense,
                driver_lincense_aquisition_fees,
            });
        }
        expenses.push(FamilyExpense {
            car_expense,
            house_expense,
            food_expense: base_food_expense,
            fuel_light_water_gas_etc_expense: base_fuel_light_water_gas_etc_expense,
            furniture_expense: base_furniture_expense,
            member_expenses,
        });
    };

    expenses
}

