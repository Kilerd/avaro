import { ActionIcon, Group, Table } from '@mantine/core';
import * as React from 'react';
import { IconChevronDown, IconChevronRight } from '@tabler/icons-react';
import { useLocalStorage } from '@mantine/hooks';
import { BudgetListItem } from '../../rest-model';
import { Buffer } from 'buffer';
import BudgetLine from './BudgetLine';
import BigNumber from 'bignumber.js';
import Amount from '../Amount';
import BackgroundProgress from '../basic/BackgroundProgress';

interface Props {
  name: string;
  items: BudgetListItem[];
}

export default function BudgetCategory(props: Props) {
  const [isShow, setCollapse] = useLocalStorage({
    key: `budget-category-${Buffer.from(props.name).toString('base64')}-collapse`,
    defaultValue: true,
  });
  const assigned_amount = props.items.reduce(
    (accr, item) => ({
      number: accr.number.plus(new BigNumber(item.assigned_amount.number)),
      commodity: item.assigned_amount.currency,
    }),
    { number: new BigNumber(0), commodity: '' },
  );

  const activity_amount = props.items.reduce(
    (accr, item) => ({
      number: accr.number.plus(new BigNumber(item.activity_amount.number)),
      commodity: item.activity_amount.currency,
    }),
    { number: new BigNumber(0), commodity: '' },
  );

  const available_amount = props.items.reduce(
    (accr, item) => ({
      number: accr.number.plus(new BigNumber(item.available_amount.number)),
      commodity: item.available_amount.currency,
    }),
    { number: new BigNumber(0), commodity: '' },
  );
  let number = BigNumber.minimum(
    new BigNumber(activity_amount.number).div(new BigNumber(assigned_amount.number)).multipliedBy(100),
    new BigNumber(100),
  ).toFormat(2);

  return (
    <>
      <TableRow style={{ position: 'relative', zIndex: 1 }}>
        <TableCell>
          <BackgroundProgress percentage={number} />
          <Group gap={'xs'}>
            <ActionIcon size="sm" color="gray" variant="transparent" onClick={() => setCollapse(!isShow)}>
              {isShow ? <IconChevronDown size={28} /> : <IconChevronRight size={48} />}
            </ActionIcon>{' '}
            <b>{props.name}</b>
          </Group>
        </TableCell>
        <TableCell style={{ textAlign: 'end' }}>
          <b>{number} %</b>
        </TableCell>
        <TableCell style={{ textAlign: 'end' }}>
          <b>
            <Amount amount={assigned_amount.number} currency={assigned_amount.commodity} />
          </b>
        </TableCell>
        <TableCell style={{ textAlign: 'end' }}>
          <b>
            <Amount amount={activity_amount.number} currency={activity_amount.commodity} />
          </b>
        </TableCell>
        <TableCell style={{ textAlign: 'end' }}>
          <b>
            <Amount amount={available_amount.number} currency={available_amount.commodity} />
          </b>
        </TableCell>
      </TableRow>
      {isShow && props.items.sort().map((item) => <BudgetLine key={`${item.name}`} {...item}></BudgetLine>)}
    </>
  );
}
