import { useRef } from 'react';

interface Props {
  value: number | null;
}

export function Money({ value }: Props) {
  const formatter = useRef(
    new Intl.NumberFormat('nl-NL', {
      currency: 'EUR',
      style: 'currency',
    }),
  );

  if (value === null) {
    return null;
  }

  return <>{formatter.current.format(value)}</>;
}
