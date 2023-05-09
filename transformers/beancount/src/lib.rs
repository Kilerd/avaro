use chrono::NaiveDate;
use itertools::{Either, Itertools};
use latestmap::LatestMap;
use std::collections::HashMap;
use std::path::PathBuf;
use zhang_ast::{Account, Balance, BalanceCheck, BalancePad, Date, Directive, Spanned};
use zhang_core::transform::TextFileBasedTransformer;
use zhang_core::{ZhangError, ZhangResult};

#[allow(clippy::upper_case_acronyms)]
#[allow(clippy::type_complexity)]
pub mod parser;

pub mod directives;

pub use crate::directives::{BeancountDirective, BeancountOnlyDirective};
use crate::parser::parse_time;
pub use parser::parse;

#[derive(Clone, Default)]
pub struct BeancountTransformer {}

macro_rules! extract_time {
    ($directive: tt) => {{
        let time = $directive
            .meta
            .pop_one("time")
            .and_then(|it| parse_time(it.as_str()).ok());
        if let Some(time) = time {
            $directive.date = Date::Datetime($directive.date.naive_date().and_time(time));
        }
    }};
}

impl BeancountTransformer {
    fn extract_time_from_meta(&self, directive: &mut BeancountDirective) {
        match directive {
            Either::Left(zhang_directive) => match zhang_directive {
                Directive::Open(directive) => extract_time!(directive),
                Directive::Close(directive) => extract_time!(directive),
                Directive::Commodity(directive) => extract_time!(directive),
                Directive::Transaction(directive) => extract_time!(directive),
                Directive::Balance(directive) => match directive {
                    Balance::BalanceCheck(balance_check) => extract_time!(balance_check),
                    Balance::BalancePad(balance_pad) => extract_time!(balance_pad),
                },
                Directive::Note(directive) => extract_time!(directive),
                Directive::Document(directive) => extract_time!(directive),
                Directive::Price(directive) => extract_time!(directive),
                Directive::Event(directive) => extract_time!(directive),
                Directive::Custom(directive) => extract_time!(directive),
                _ => {}
            },
            Either::Right(beancount_onyly_directive) => match beancount_onyly_directive {
                BeancountOnlyDirective::Pad(directive) => extract_time!(directive),
                BeancountOnlyDirective::Balance(directive) => extract_time!(directive),
                _ => {}
            },
        }
    }
}

impl TextFileBasedTransformer for BeancountTransformer {
    type FileOutput = Spanned<BeancountDirective>;

    fn parse(&self, content: &str, path: PathBuf) -> ZhangResult<Vec<Self::FileOutput>> {
        parse(content, path).map_err(|it| ZhangError::PestError(it.to_string()))
    }

    fn go_next(&self, directive: &Self::FileOutput) -> Option<String> {
        match &directive.data {
            Either::Left(Directive::Include(include)) => Some(include.file.clone().to_plain_string()),
            _ => None,
        }
    }
    fn transform(&self, directives: Vec<Self::FileOutput>) -> ZhangResult<Vec<Spanned<Directive>>> {
        let mut ret = vec![];
        let mut tags_stack: Vec<String> = vec![];

        let mut pad_info: LatestMap<NaiveDate, HashMap<String, Account>> = LatestMap::default();

        for directives in directives {
            let Spanned { span, mut data } = directives;
            self.extract_time_from_meta(&mut data);
            match data {
                Either::Left(zhang_directive) => match zhang_directive {
                    Directive::Transaction(mut trx) => {
                        for tag in &tags_stack {
                            trx.tags.insert(tag.to_owned());
                        }
                        ret.push(Spanned {
                            span,
                            data: Directive::Transaction(trx),
                        });
                    }
                    _ => ret.push(Spanned {
                        span,
                        data: zhang_directive,
                    }),
                },
                Either::Right(beancount_directive) => match beancount_directive {
                    BeancountOnlyDirective::PushTag(tag) => tags_stack.push(tag),
                    BeancountOnlyDirective::PopTag(tag) => {
                        tags_stack = tags_stack.into_iter().filter(|it| it.ne(&tag)).collect_vec()
                    }
                    BeancountOnlyDirective::Pad(pad) => {
                        let date = pad.date.naive_date();
                        if !pad_info.contains_key(&date) {
                            pad_info.insert(date, HashMap::new());
                        }
                        let target_date_pad_info = pad_info.get_mut(&date).expect("pad info must contains the key");
                        target_date_pad_info.insert(pad.account.content, pad.pad);
                    }
                    BeancountOnlyDirective::Balance(balance) => {
                        let date = balance.date.naive_date();
                        let pad_account = pad_info
                            .get_latest(&date)
                            .and_then(|it| it.get(&balance.account.content));

                        if let Some(pad_account) = pad_account {
                            // balance pad
                            ret.push(Spanned {
                                span,
                                data: Directive::Balance(Balance::BalancePad(BalancePad {
                                    date: balance.date,
                                    account: balance.account,
                                    amount: balance.amount,
                                    pad: pad_account.clone(),
                                    meta: balance.meta,
                                })),
                            });
                        } else {
                            //balance check
                            ret.push(Spanned {
                                span,
                                data: Directive::Balance(Balance::BalanceCheck(BalanceCheck {
                                    date: balance.date,
                                    account: balance.account,
                                    amount: balance.amount,
                                    meta: balance.meta,
                                })),
                            });
                        }
                    }
                },
            }
        }
        Ok(ret)
    }
}

#[cfg(test)]
mod test {
    use crate::directives::{BalanceDirective, BeancountDirective, BeancountOnlyDirective, PadDirective};
    use crate::BeancountTransformer;
    use bigdecimal::BigDecimal;
    use chrono::NaiveDate;
    use std::str::FromStr;
    use zhang_ast::amount::Amount;
    use zhang_ast::{
        Account, Balance, BalanceCheck, BalancePad, Date, Directive, Meta, Open, SpanInfo, Spanned, Transaction,
        ZhangString,
    };
    use zhang_core::transform::TextFileBasedTransformer;

    fn fake_span() -> SpanInfo {
        SpanInfo {
            start: 0,
            end: 0,
            content: "".to_string(),
            filename: None,
        }
    }

    #[test]
    fn should_append_tag_to_transaction_directive_given_push_tag_directive() {
        let transformer = BeancountTransformer::default();
        let mut directives = transformer
            .transform(vec![
                Spanned::new(
                    BeancountDirective::Right(BeancountOnlyDirective::PushTag("onetag".to_string())),
                    fake_span(),
                ),
                Spanned::new(
                    BeancountDirective::Left(Directive::Transaction(Transaction {
                        date: Date::Date(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                        flag: None,
                        payee: None,
                        narration: None,
                        tags: Default::default(),
                        links: Default::default(),
                        postings: vec![],
                        meta: Default::default(),
                    })),
                    fake_span(),
                ),
            ])
            .unwrap();

        assert_eq!(directives.len(), 1);
        let directive = directives.pop().unwrap().data;
        match directive {
            Directive::Transaction(mut trx) => assert_eq!("onetag", trx.tags.pop().unwrap()),
            _ => unreachable!(),
        }
    }

    #[test]
    fn should_not_append_tag_to_transaction_directive_given_push_tag_directive() {
        let transformer = BeancountTransformer::default();
        let mut directives = transformer
            .transform(vec![
                Spanned::new(
                    BeancountDirective::Right(BeancountOnlyDirective::PushTag("onetag".to_string())),
                    fake_span(),
                ),
                Spanned::new(
                    BeancountDirective::Right(BeancountOnlyDirective::PopTag("onetag".to_string())),
                    fake_span(),
                ),
                Spanned::new(
                    BeancountDirective::Left(Directive::Transaction(Transaction {
                        date: Date::Date(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                        flag: None,
                        payee: None,
                        narration: None,
                        tags: Default::default(),
                        links: Default::default(),
                        postings: vec![],
                        meta: Default::default(),
                    })),
                    fake_span(),
                ),
            ])
            .unwrap();

        assert_eq!(directives.len(), 1);
        let directive = directives.pop().unwrap().data;
        match directive {
            Directive::Transaction(mut trx) => assert_eq!(None, trx.tags.pop()),
            _ => unreachable!(),
        }
    }

    #[test]
    fn should_transform_to_non_given_pad_directive() {
        let transformer = BeancountTransformer::default();
        let directives = transformer
            .transform(vec![Spanned::new(
                BeancountDirective::Right(BeancountOnlyDirective::Pad(PadDirective {
                    date: Date::Date(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                    account: Account::from_str("Assets::BankAccount").unwrap(),
                    pad: Account::from_str("Equity::Open-Balances").unwrap(),
                    meta: Default::default(),
                })),
                fake_span(),
            )])
            .unwrap();

        assert_eq!(directives.len(), 0);
    }

    #[test]
    fn should_transform_to_balance_check_directive_given_balance_directive() {
        let transformer = BeancountTransformer::default();
        let mut directives = transformer
            .transform(vec![Spanned::new(
                BeancountDirective::Right(BeancountOnlyDirective::Balance(BalanceDirective {
                    date: Date::Date(NaiveDate::from_ymd_opt(1970, 1, 2).unwrap()),
                    account: Account::from_str("Assets::BankAccount").unwrap(),
                    meta: Default::default(),
                    amount: Amount::new(BigDecimal::from(100i32), "CNY"),
                })),
                fake_span(),
            )])
            .unwrap();

        assert_eq!(directives.len(), 1);

        let balance_pad_directive = directives.pop().unwrap().data;

        assert_eq!(
            balance_pad_directive,
            Directive::Balance(Balance::BalanceCheck(BalanceCheck {
                date: Date::Date(NaiveDate::from_ymd_opt(1970, 1, 2).unwrap()),
                account: Account::from_str("Assets::BankAccount").unwrap(),
                amount: Amount::new(BigDecimal::from(100i32), "CNY"),
                meta: Default::default(),
            }))
        );
    }

    #[test]
    fn should_transform_to_balance_pad_directive_given_pad_and_balance_directive() {
        let transformer = BeancountTransformer::default();
        let mut directives = transformer
            .transform(vec![
                Spanned::new(
                    BeancountDirective::Right(BeancountOnlyDirective::Pad(PadDirective {
                        date: Date::Date(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                        account: Account::from_str("Assets::BankAccount").unwrap(),
                        pad: Account::from_str("Equity::Open-Balances").unwrap(),
                        meta: Default::default(),
                    })),
                    fake_span(),
                ),
                Spanned::new(
                    BeancountDirective::Right(BeancountOnlyDirective::Balance(BalanceDirective {
                        date: Date::Date(NaiveDate::from_ymd_opt(1970, 1, 2).unwrap()),
                        account: Account::from_str("Assets::BankAccount").unwrap(),
                        meta: Default::default(),
                        amount: Amount::new(BigDecimal::from(100i32), "CNY"),
                    })),
                    fake_span(),
                ),
            ])
            .unwrap();

        assert_eq!(directives.len(), 1);

        let balance_pad_directive = directives.pop().unwrap().data;

        assert_eq!(
            balance_pad_directive,
            Directive::Balance(Balance::BalancePad(BalancePad {
                date: Date::Date(NaiveDate::from_ymd_opt(1970, 1, 2).unwrap()),
                account: Account::from_str("Assets::BankAccount").unwrap(),
                amount: Amount::new(BigDecimal::from(100i32), "CNY"),
                pad: Account::from_str("Equity::Open-Balances").unwrap(),
                meta: Default::default(),
            }))
        );
    }

    #[test]
    fn should_parse_time_from_meta() {
        let transformer = BeancountTransformer::default();

        let mut meta = Meta::default();
        meta.insert("time".to_string(), ZhangString::quote("01:02:03"));
        let mut directives = transformer
            .transform(vec![Spanned::new(
                BeancountDirective::Left(Directive::Open(Open {
                    date: Date::Date(NaiveDate::from_ymd_opt(1970, 1, 2).unwrap()),
                    account: Account::from_str("Assets::BankAccount").unwrap(),
                    commodities: vec![],
                    meta,
                })),
                fake_span(),
            )])
            .unwrap();

        assert_eq!(directives.len(), 1);

        let balance_pad_directive = directives.pop().unwrap().data;

        assert_eq!(
            balance_pad_directive,
            Directive::Open(Open {
                date: Date::Datetime(
                    NaiveDate::from_ymd_opt(1970, 1, 2)
                        .unwrap()
                        .and_hms_micro_opt(1, 2, 3, 0)
                        .unwrap()
                ),
                account: Account::from_str("Assets::BankAccount").unwrap(),
                commodities: vec![],
                meta: Meta::default(),
            })
        );
    }
}
