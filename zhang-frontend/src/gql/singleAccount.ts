import { gql } from "@apollo/client";
import { AccountItem } from "./accountList";


export interface SingleAccountJournalQuery {
    account: AccountItem
}


export const SINGLE_ACCONT_JOURNAL = gql`
query SINGLE_ACCONT_JOURNAL($name: String) {
    account(name: $name) {
        name
        status
        currencies {
          name
        }
        snapshot {
          detail {
            number
            currency
          }
        }
        documents {
          filename
          __typename
        }
        journals {
            date
            type: __typename
            ... on TransactionDto {
              payee
              narration
              postings {
                account {
                  name
                }
                unit {
                  number
                  currency
                }
              }
            }
            ... on BalanceCheckDto {
              account {
                name
              }
              balanceAmount {
                number
                currency
              }
              currentAmount {
                number
                currency
              }
              isBalanced
              distance {
                number
                currency
              }
            }
          }
    }
  }    
`