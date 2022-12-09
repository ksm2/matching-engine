import { Money } from './Money';

interface Props {
  color: 'red' | 'green';
  price: number;
  qty: number;
}
export function OrderBookRow({ color, price, qty }: Props) {
  return (
    <tr style={{ color: color === 'red' ? '#f65555' : '#62a862' }}>
      <td>{qty}</td>
      <td style={{ textAlign: 'right' }}>
        <Money value={price} />
      </td>
    </tr>
  );
}
