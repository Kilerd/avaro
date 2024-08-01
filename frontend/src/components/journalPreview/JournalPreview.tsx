import { JournalBalanceCheckItem, JournalBalancePadItem, JournalItem, JournalTransactionItem } from '../../rest-model';
import BalanceCheckPreview from './BalanceCheckPreview';
import BalancePadPreview from './BalancePadPreview';
import TransactionPreview from './TransactionPreview';

interface Props {
  data?: JournalItem;
}

export default function JournalPreview(props: Props) {
  let line = null;
  if (!props.data) {
    return <div>preview click</div>;
  }
  switch (props.data.type) {
    case 'BalanceCheck':
      line = <BalanceCheckPreview data={props.data as JournalBalanceCheckItem} />;
      break;
    case 'BalancePad':
      line = <BalancePadPreview data={props.data as JournalBalancePadItem} />;
      break;
    case 'Transaction':
      line = <TransactionPreview data={props.data as JournalTransactionItem} />;
      break;
  }
  return line;
}
