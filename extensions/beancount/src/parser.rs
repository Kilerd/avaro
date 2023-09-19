use std::path::PathBuf;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveTime};
use itertools::{Either, Itertools};
use pest_consume::{match_nodes, Error, Parser};
use snailquote::unescape;
use zhang_ast::amount::Amount;
use zhang_ast::utils::multi_value_map::MultiValueMap;
use zhang_ast::*;

use crate::directives::{BalanceDirective, BeancountDirective, BeancountOnlyDirective, PadDirective};

type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[derive(Parser)]
#[grammar = "beancount.pest"]
pub struct BeancountParer;

#[pest_consume::parser]
impl BeancountParer {
    #[allow(dead_code)]
    fn EOI(_input: Node) -> Result<()> {
        Ok(())
    }
    fn number(input: Node) -> Result<BigDecimal> {
        Ok(BigDecimal::from_str(input.as_str()).unwrap())
    }
    fn quote_string(input: Node) -> Result<ZhangString> {
        let string = input.as_str();
        Ok(ZhangString::QuoteString(unescape(string).unwrap()))
    }

    fn unquote_string(input: Node) -> Result<ZhangString> {
        Ok(ZhangString::UnquoteString(input.as_str().to_owned()))
    }

    fn string(input: Node) -> Result<ZhangString> {
        let ret = match_nodes!(
            input.into_children();
            [quote_string(i)] => i,
            [unquote_string(i)] => i
        );
        Ok(ret)
    }
    fn commodity_name(input: Node) -> Result<String> {
        Ok(input.as_str().to_owned())
    }
    fn account_type(input: Node) -> Result<String> {
        Ok(input.as_str().to_owned())
    }
    fn account_name(input: Node) -> Result<Account> {
        let r: (String, Vec<String>) = match_nodes!(input.into_children();
            [account_type(a), unquote_string(i)..] => {
                (a, i.map(|it|it.to_plain_string()).collect())
            },

        );
        Ok(Account {
            account_type: AccountType::from_str(&r.0).unwrap(),
            content: format!("{}:{}", &r.0, r.1.join(":")),
            components: r.1,
        })
    }
    fn date(input: Node) -> Result<Date> {
        let datetime: Date = match_nodes!(input.into_children();
            [date_only(d)] => d,
        );
        Ok(datetime)
    }

    fn date_only(input: Node) -> Result<Date> {
        let date = NaiveDate::parse_from_str(input.as_str(), "%Y-%m-%d").unwrap();
        Ok(Date::Date(date))
    }

    fn plugin(input: Node) -> Result<Directive> {
        let ret: (ZhangString, Vec<ZhangString>) = match_nodes!(input.into_children();
            [string(module), string(values)..] => (module, values.collect()),
        );
        Ok(Directive::Plugin(Plugin { module: ret.0, value: ret.1 }))
    }

    fn option(input: Node) -> Result<Directive> {
        let (key, value) = match_nodes!(input.into_children();
            [string(key), string(value)] => (key, value),
        );
        Ok(Directive::Option(Options { key, value }))
    }
    fn comment(input: Node) -> Result<Directive> {
        Ok(Directive::Comment(Comment {
            content: input.as_str().to_owned(),
        }))
    }

    fn open(input: Node) -> Result<Directive> {
        let ret: (Date, Account, Vec<String>, Vec<(String, ZhangString)>) = match_nodes!(input.into_children();
            [date(date), account_name(a), commodity_name(commodities).., commodity_meta(metas)] => (date, a, commodities.collect(), metas),
            [date(date), account_name(a), commodity_name(commodities)..] => (date, a, commodities.collect(), vec![]),
            [date(date), account_name(a), commodity_meta(metas)] => (date, a, vec![], metas),
        );

        let open = Open {
            date: ret.0,
            account: ret.1,
            commodities: ret.2,
            meta: ret.3.into_iter().collect(),
        };
        Ok(Directive::Open(open))
    }
    fn close(input: Node) -> Result<Directive> {
        let ret: (Date, Account) = match_nodes!(input.into_children();
            [date(date), account_name(a)] => (date, a)
        );
        Ok(Directive::Close(Close {
            date: ret.0,
            account: ret.1,
            meta: Default::default(),
        }))
    }

    #[allow(dead_code)]
    fn identation(input: Node) -> Result<()> {
        Ok(())
    }

    fn commodity_line(input: Node) -> Result<(String, ZhangString)> {
        let ret: (String, ZhangString) = match_nodes!(input.into_children();
            [string(key), string(value)] => (key.to_plain_string(), value),
        );
        Ok(ret)
    }

    fn commodity_meta(input: Node) -> Result<Vec<(String, ZhangString)>> {
        let ret: Vec<(String, ZhangString)> = match_nodes!(input.into_children();
            [commodity_line(lines)..] => lines.collect(),
        );
        Ok(ret)
    }

    fn posting_unit(input: Node) -> Result<(Option<Amount>, Option<(Option<Amount>, Option<Date>, Option<SingleTotalPrice>)>)> {
        let ret: (Option<Amount>, Option<(Option<Amount>, Option<Date>, Option<SingleTotalPrice>)>) = match_nodes!(input.into_children();
            [posting_amount(amount)] => (Some(amount), None),
            [posting_meta(meta)] => (None, Some(meta)),
            [posting_amount(amount), posting_meta(meta)] => (Some(amount), Some(meta)),
        );
        Ok(ret)
    }

    fn posting_cost(input: Node) -> Result<Amount> {
        let ret: Amount = match_nodes!(input.into_children();
            [number(amount), commodity_name(c)] => Amount::new(amount, c),
        );
        Ok(ret)
    }
    fn posting_total_price(input: Node) -> Result<Amount> {
        let ret: Amount = match_nodes!(input.into_children();
            [number(amount), commodity_name(c)] => Amount::new(amount, c),
        );
        Ok(ret)
    }
    fn posting_single_price(input: Node) -> Result<Amount> {
        let ret: Amount = match_nodes!(input.into_children();
            [number(amount), commodity_name(c)] => Amount::new(amount, c),
        );
        Ok(ret)
    }

    fn posting_amount(input: Node) -> Result<Amount> {
        let ret: Amount = match_nodes!(input.into_children();
            [number(amount), commodity_name(c)] => Amount::new(amount, c),
        );
        Ok(ret)
    }

    fn transaction_flag(input: Node) -> Result<Option<Flag>> {
        Ok(Some(Flag::from_str(input.as_str().trim()).unwrap()))
    }

    fn posting_price(input: Node) -> Result<SingleTotalPrice> {
        let ret: SingleTotalPrice = match_nodes!(input.into_children();
            [posting_total_price(p)] => SingleTotalPrice::Total(p),
            [posting_single_price(p)] => SingleTotalPrice::Single(p),
        );
        Ok(ret)
    }
    fn posting_meta(input: Node) -> Result<(Option<Amount>, Option<Date>, Option<SingleTotalPrice>)> {
        let ret: (Option<Amount>, Option<Date>, Option<SingleTotalPrice>) = match_nodes!(input.into_children();
            [] => (None, None, None),
            [posting_cost(cost)] => (Some(cost), None, None),
            [posting_price(p)] => (None, None, Some(p)),
            [posting_cost(cost), date(d)] => (Some(cost), Some(d), None),
            [posting_cost(cost), posting_price(p)] => (Some(cost), None, Some(p)),
            [posting_cost(cost), date(d), posting_price(p)] => (Some(cost), Some(d), Some(p)),
        );
        Ok(ret)
    }
    fn transaction_posting(input: Node) -> Result<Posting> {
        let ret: (
            Option<Flag>,
            Account,
            Option<(Option<Amount>, Option<(Option<Amount>, Option<Date>, Option<SingleTotalPrice>)>)>,
        ) = match_nodes!(input.into_children();
            [account_name(account_name)] => (None, account_name, None),
            [account_name(account_name), posting_unit(unit)] => (None, account_name, Some(unit)),
            [transaction_flag(flag), account_name(account_name)] => (flag, account_name, None),
            [transaction_flag(flag), account_name(account_name), posting_unit(unit)] => (flag, account_name, Some(unit)),
        );

        let (flag, account, unit) = ret;

        let mut line = Posting {
            flag,
            account,
            units: None,
            cost: None,
            cost_date: None,
            price: None,
            meta: Default::default(),
        };

        if let Some((amount, meta)) = unit {
            line.units = amount;

            if let Some(meta) = meta {
                line.cost = meta.0;
                line.cost_date = meta.1;
                line.price = meta.2;
            }
        }
        Ok(line)
    }

    fn transaction_line(input: Node) -> Result<(Option<Posting>, Option<(String, ZhangString)>)> {
        let ret: (Option<Posting>, Option<(String, ZhangString)>) = match_nodes!(input.into_children();
            [transaction_posting(posting)] => (Some(posting), None),
            [commodity_line(meta)] => (None, Some(meta)),

        );
        Ok(ret)
    }
    fn transaction_lines(input: Node) -> Result<Vec<(Option<Posting>, Option<(String, ZhangString)>)>> {
        let ret = match_nodes!(input.into_children();
            [transaction_line(lines)..] => lines.collect(),
        );
        Ok(ret)
    }

    fn tag(input: Node) -> Result<String> {
        let ret = match_nodes!(input.into_children();
            [unquote_string(tag)] => tag.to_plain_string(),
        );
        Ok(ret)
    }
    fn link(input: Node) -> Result<String> {
        let ret = match_nodes!(input.into_children();
            [unquote_string(tag)] => tag.to_plain_string(),
        );
        Ok(ret)
    }
    fn tags(input: Node) -> Result<Vec<String>> {
        let ret = match_nodes!(input.into_children();
            [tag(tags)..] => tags.collect(),
        );
        Ok(ret)
    }
    fn links(input: Node) -> Result<Vec<String>> {
        let ret = match_nodes!(input.into_children();
            [link(links)..] => links.collect(),
        );
        Ok(ret)
    }

    fn transaction(input: Node) -> Result<Directive> {
        let ret: (
            Date,
            Option<Flag>,
            Option<ZhangString>,
            Option<ZhangString>,
            Vec<String>,
            Vec<String>,
            Vec<(Option<Posting>, Option<(String, ZhangString)>)>,
        ) = match_nodes!(input.into_children();
            [date(date), quote_string(payee), tags(tags), links(links), transaction_lines(lines)] => (date, None, Some(payee), None, tags, links,lines),
            [date(date), quote_string(payee), quote_string(narration), tags(tags), links(links), transaction_lines(lines)] => (date, None, Some(payee), Some(narration), tags, links,lines),
            [date(date), transaction_flag(flag), tags(tags), links(links), transaction_lines(lines)] => (date, flag, None, None, tags, links, lines),
            [date(date), transaction_flag(flag), quote_string(narration), tags(tags), links(links), transaction_lines(lines)] => (date, flag, None, Some(narration), tags, links, lines),
            [date(date), transaction_flag(flag), quote_string(payee), quote_string(narration), tags(tags), links(links), transaction_lines(lines)] => (date, flag, Some(payee), Some(narration), tags, links,lines),
        );
        let mut transaction = Transaction {
            date: ret.0,
            flag: ret.1,
            payee: ret.2,
            narration: ret.3,
            tags: ret.4.into_iter().collect(),
            links: ret.5.into_iter().collect(),
            postings: vec![],
            meta: MultiValueMap::default(),
        };

        for line in ret.6 {
            match line {
                (Some(trx), None) => {
                    transaction.postings.push(trx);
                }
                (None, Some(meta)) => {
                    transaction.meta.insert(meta.0, meta.1);
                }
                _ => {}
            }
        }

        Ok(Directive::Transaction(transaction))
    }

    fn commodity(input: Node) -> Result<Directive> {
        let ret = match_nodes!(input.into_children();
            [date(date), commodity_name(name)] => (date, name, vec![]),
            [date(date), commodity_name(name), commodity_meta(meta)] => (date, name, meta),
        );
        Ok(Directive::Commodity(Commodity {
            date: ret.0,
            currency: ret.1,
            meta: ret.2.into_iter().collect(),
        }))
    }

    fn string_or_account(input: Node) -> Result<StringOrAccount> {
        let ret: StringOrAccount = match_nodes!(input.into_children();
            [string(value)] => StringOrAccount::String(value),
            [account_name(value)] => StringOrAccount::Account(value),
        );
        Ok(ret)
    }

    fn custom(input: Node) -> Result<Directive> {
        let ret: (Date, ZhangString, Vec<StringOrAccount>) = match_nodes!(input.into_children();
            [date(date), string(module), string_or_account(options)..] => (date, module, options.collect()),
        );
        Ok(Directive::Custom(Custom {
            date: ret.0,
            custom_type: ret.1,
            values: ret.2,
            meta: Default::default(),
        }))
    }

    fn include(input: Node) -> Result<Directive> {
        let ret: ZhangString = match_nodes!(input.into_children();
            [quote_string(path)] => path,
        );
        let include = Include { file: ret };
        Ok(Directive::Include(include))
    }

    fn note(input: Node) -> Result<Directive> {
        let ret: (Date, Account, ZhangString) = match_nodes!(input.into_children();
            [date(date), account_name(a), string(path)] => (date, a, path),
        );
        Ok(Directive::Note(Note {
            date: ret.0,
            account: ret.1,
            comment: ret.2,
            tags: None,
            links: None,
            meta: Default::default(),
        }))
    }

    fn event(input: Node) -> Result<Directive> {
        let ret: (Date, ZhangString, ZhangString) = match_nodes!(input.into_children();
            [date(date), string(name), string(value)] => (date, name, value),
        );
        Ok(Directive::Event(Event {
            date: ret.0,
            event_type: ret.1,
            description: ret.2,
            meta: Default::default(),
        }))
    }

    fn balance(input: Node) -> Result<BeancountOnlyDirective> {
        let (date, account, amount, commodity): (Date, Account, BigDecimal, String) = match_nodes!(input.into_children();
            [date(date), account_name(name), number(amount), commodity_name(commodity)] => (date, name, amount, commodity),
        );
        Ok(BeancountOnlyDirective::Balance(BalanceDirective {
            date,
            account,
            amount: Amount::new(amount, commodity),
            meta: Default::default(),
        }))
    }
    fn pad(input: Node) -> Result<BeancountOnlyDirective> {
        let (date, name, pad): (Date, Account, Account) = match_nodes!(input.into_children();
            [date(date), account_name(name), account_name(pad)] => (date, name, pad),
        );
        Ok(BeancountOnlyDirective::Pad(PadDirective {
            date,
            account: name,
            pad,
            meta: Default::default(),
        }))
    }

    fn document(input: Node) -> Result<Directive> {
        let ret: (Date, Account, ZhangString) = match_nodes!(input.into_children();
            [date(date), account_name(name), string(path)] => (date, name, path),
        );
        Ok(Directive::Document(Document {
            date: ret.0,
            account: ret.1,
            filename: ret.2,
            tags: None,
            links: None,
            meta: Default::default(),
        }))
    }

    fn price(input: Node) -> Result<Directive> {
        let ret: (Date, String, BigDecimal, String) = match_nodes!(input.into_children();
            [date(date), commodity_name(source), number(price), commodity_name(target)] => (date, source, price, target)
        );
        Ok(Directive::Price(Price {
            date: ret.0,
            currency: ret.1,
            amount: Amount::new(ret.2, ret.3),
            meta: Default::default(),
        }))
    }
    fn push_tag(input: Node) -> Result<BeancountOnlyDirective> {
        let ret: String = match_nodes!(input.into_children();
            [tag(tag)] => tag
        );
        Ok(BeancountOnlyDirective::PushTag(ret))
    }
    fn pop_tag(input: Node) -> Result<BeancountOnlyDirective> {
        let ret: String = match_nodes!(input.into_children();
            [tag(tag)] => tag
        );
        Ok(BeancountOnlyDirective::PopTag(ret))
    }

    fn item(input: Node) -> Result<(BeancountDirective, SpanInfo)> {
        let span = input.as_span();
        let span_info = SpanInfo {
            start: span.start_pos().pos(),
            end: span.end_pos().pos(),
            content: span.as_str().to_string(),
            filename: None,
        };
        let ret: BeancountDirective = match_nodes!(input.into_children();
            [option(item)]      => Either::Left(item),
            [open(item)]        => Either::Left(item),
            [plugin(item)]      => Either::Left(item),
            [close(item)]       => Either::Left(item),
            [include(item)]     => Either::Left(item),
            [note(item)]        => Either::Left(item),
            [event(item)]       => Either::Left(item),
            [document(item)]    => Either::Left(item),
            [balance(item)]     => Either::Right(item), // balance
            [pad(item)]         => Either::Right(item), // pad
            [push_tag(item)]    => Either::Right(item),
            [pop_tag(item)]     => Either::Right(item),
            [price(item)]       => Either::Left(item),
            [commodity(item)]   => Either::Left(item),
            [custom(item)]      => Either::Left(item),
            [comment(item)]     => Either::Left(item),
            [transaction(item)] => Either::Left(item),
        );
        Ok((ret, span_info))
    }

    fn time_part(input: Node) -> Result<u32> {
        Ok(u32::from_str(input.as_str()).unwrap())
    }

    fn time(input: Node) -> Result<NaiveTime> {
        let (hour, min, sec): (u32, u32, u32) = match_nodes!(input.into_children();
            [time_part(hour), time_part(min), time_part(sec)] => (hour, min, sec),
        );
        Ok(NaiveTime::from_hms_opt(hour, min, sec).expect("not a valid time"))
    }

    fn entry(input: Node) -> Result<Vec<Spanned<BeancountDirective>>> {
        let ret: Vec<(BeancountDirective, SpanInfo)> = match_nodes!(input.into_children();
            [item(items).., _] => items.collect(),
        );
        Ok(ret
            .into_iter()
            .map(|(directive, span_info)| Spanned {
                data: directive,
                span: span_info,
            })
            .collect_vec())
    }
}

pub fn parse(input_str: &str, file: impl Into<Option<PathBuf>>) -> Result<Vec<Spanned<BeancountDirective>>> {
    let file = file.into();
    let inputs = BeancountParer::parse(Rule::entry, input_str)?;
    let input = inputs.single()?;
    BeancountParer::entry(input).map(|mut directives| {
        directives.iter_mut().for_each(|directive| directive.span.filename = file.clone());
        directives
    })
}
pub fn parse_time(input_str: &str) -> Result<NaiveTime> {
    let inputs = BeancountParer::parse(Rule::time, input_str)?;
    let input = inputs.single()?;
    BeancountParer::time(input)
}

#[cfg(test)]
mod test {

    mod tag {
        use std::str::FromStr;

        use bigdecimal::BigDecimal;
        use chrono::NaiveDate;
        use zhang_ast::amount::Amount;
        use zhang_ast::{Account, Date};

        use crate::directives::{BalanceDirective, BeancountOnlyDirective, PadDirective};
        use crate::parser::parse;

        #[test]
        fn should_support_push_tag() {
            let directive = parse("pushtag #mytag", None).unwrap().pop().unwrap().data.right().unwrap();
            assert_eq!(BeancountOnlyDirective::PushTag("mytag".to_string()), directive);
        }
        #[test]
        fn should_support_pop_tag() {
            let directive = parse("poptag #mytag", None).unwrap().pop().unwrap().data.right().unwrap();
            assert_eq!(BeancountOnlyDirective::PopTag("mytag".to_string()), directive);
        }

        #[test]
        fn should_parse_balance() {
            let directive = parse("1970-01-01 balance Assets:BankAccount 2 CNY", None)
                .unwrap()
                .pop()
                .unwrap()
                .data
                .right()
                .unwrap();
            assert_eq!(
                BeancountOnlyDirective::Balance(BalanceDirective {
                    date: Date::Date(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                    account: Account::from_str("Assets:BankAccount").unwrap(),
                    amount: Amount::new(BigDecimal::from(2i32), "CNY"),
                    meta: Default::default(),
                }),
                directive
            );
        }
        #[test]
        fn should_parse_pad() {
            let directive = parse("1970-01-01 pad Assets:BankAccount Assets:BankAccount2", None)
                .unwrap()
                .pop()
                .unwrap()
                .data
                .right()
                .unwrap();
            assert_eq!(
                BeancountOnlyDirective::Pad(PadDirective {
                    date: Date::Date(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                    account: Account::from_str("Assets:BankAccount").unwrap(),
                    pad: Account::from_str("Assets:BankAccount2").unwrap(),
                    meta: Default::default(),
                }),
                directive
            );
        }
    }
}
