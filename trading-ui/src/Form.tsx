import { Dispatch, useState } from 'react';
import { Order } from './model';

interface Props {
  midPrice: number | null;
  onOrder: Dispatch<Order>;
}

export function Form({ midPrice, onOrder }: Props) {
  const [side, setSide] = useState('Buy');
  const [price, setPrice] = useState(midPrice ?? 0);
  const [quantity, setQuantity] = useState(100);

  async function handleOrder() {
    const body = JSON.stringify({ side, price, quantity });
    const response = await fetch('/api/orders', { method: 'POST', body });
    console.log(response.status);
  }

  return (
    <div className="Form">
      <div className="FormRow">
        <label htmlFor="frm-side">Side</label>
        <select value={side} id="frm-side" onChange={(e) => setSide(e.target.value)}>
          <option>Buy</option>
          <option>Sell</option>
        </select>
      </div>
      <div className="FormRow">
        <label htmlFor="frm-price">Price</label>
        <input
          type="number"
          id="frm-price"
          value={price}
          style={{ textAlign: 'right' }}
          onChange={(e) => setPrice(e.target.valueAsNumber)}
        />
      </div>
      <div className="FormRow">
        <label htmlFor="frm-qty">Qty</label>
        <input
          type="number"
          id="frm-qty"
          value={quantity}
          style={{ textAlign: 'right' }}
          onChange={(e) => setQuantity(e.target.valueAsNumber)}
        />
      </div>
      <button onClick={handleOrder}>Send Order</button>
    </div>
  );
}
