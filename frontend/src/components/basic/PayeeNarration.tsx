import { Group, Text } from '@mantine/core';
import { createStyles } from '@mantine/emotion';

const useStyles = createStyles((theme, _, u) => ({
  payee: {
    fontWeight: 700,
    [u.dark]: { color: theme.white },
    [u.light]: { color: theme.black },

    '&:after': {
      fontWeight: 700,
      marginLeft: theme.spacing.xs,
      content: '"·"',
    },
  },
  narration: {},
}));

interface Props {
  payee?: string;
  narration?: string;
}

export default function PayeeNarration(props: Props) {
  const { classes } = useStyles();
  return (
    <Group gap="xs">
      {props.payee && <Text className={classes.payee}>{props.payee}</Text>}
      <Text className={classes.narration}>{props.narration ?? ''}</Text>
    </Group>
  );
}
