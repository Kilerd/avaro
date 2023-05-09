import { JournalBalanceCheckItem } from '../../../rest-model';
import { ActionIcon, Badge, createStyles } from '@mantine/core';
import { format } from 'date-fns';
import Amount from '../../Amount';
import BigNumber from 'bignumber.js';
import { IconZoomExclamation } from '@tabler/icons';
import { openContextModal } from '@mantine/modals';

const useStyles = createStyles((theme, _params, getRef) => ({
  payee: {
    fontWeight: 'bold',
  },
  narration: {},
  positiveAmount: {
    color: theme.colors.gray[7],
    fontWeight: 'bold',
    fontFeatureSettings: 'tnum',
    fontSize: theme.fontSizes.sm * 0.95,
  },
  negativeAmount: {
    color: theme.colors.red[5],
    fontWeight: 'bold',
    fontFeatureSettings: 'tnum',
    fontSize: theme.fontSizes.sm,
  },
  notBalance: {
    borderLeft: '3px solid red',
  },
  wrapper: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'end',
  },
  actionHider: {
    '&:hover': {
      [`& .${getRef('actions')}`]: {
        display: 'flex',
        alignItems: 'end',
        justifyContent: 'end',
      },
    },
  },
  actions: {
    ref: getRef('actions'),
    display: 'none',
  },
}));

interface Props {
  data: JournalBalanceCheckItem;
}

export default function TableViewBalanceCheckLine({ data }: Props) {
  const { classes } = useStyles();

  const date = format(new Date(data.datetime), 'yyyy-MM-dd');
  const time = format(new Date(data.datetime), 'hh:mm:ss');

  const openPreviewModal = (e: any) => {
    openContextModal({
      modal: 'transactionPreviewModal',
      title: 'transaction preview',
      size: 'lg',
      centered: true,
      innerProps: {
        journalId: data.id,
      },
    });
  };
  const isBalanced = new BigNumber(data.postings[0].account_after_number).eq(new BigNumber(data.postings[0].account_before_number));
  return (
    <tr className={`${classes.actionHider} ${!isBalanced ? classes.notBalance : ''}`}>
      <td>
        {date} {time}
      </td>
      <td>
        <Badge size="xs" variant="outline">
          Check
        </Badge>
      </td>
      <td>{data.payee}</td>
      <td>{data.narration}</td>
      <td>
        <div className={classes.wrapper}>
          <div className={!isBalanced ? classes.negativeAmount : classes.positiveAmount}>
            <Amount amount={data.postings[0].account_after_number} currency={data.postings[0].account_after_commodity} />
          </div>
          {!isBalanced && (
            <span className={classes.positiveAmount}>
              current: <Amount amount={data.postings[0].account_before_number} currency={data.postings[0].account_before_commodity} />
            </span>
          )}
        </div>
      </td>
      <td>
        <div className={classes.actions}>
          <ActionIcon size="sm" onClick={openPreviewModal}>
            <IconZoomExclamation size="1.125rem" />
          </ActionIcon>
        </div>
      </td>
    </tr>
  );
}
