import './App.css';
import { Form } from './Form';
import { useOrderBook } from './hooks/useOrderBook';
import { Money } from './Money';
import { OrderBookRow } from './OrderBookRow';

export function App() {
  const { last, asks, bids, bestAsk, bestBid } = useOrderBook();
  const spread = bestAsk && bestBid ? bestAsk - bestBid : null;
  const midPrice = bestAsk && bestBid ? (bestAsk + bestBid) / 2 : null;

  return (
    <div className="App">
      <div className="Top">
        <div>
          <span style={{ marginRight: '1rem' }}>Last:</span>
          <Money value={last} />
        </div>
      </div>
      <div className="OrderBook">
        <table>
          <tbody>
            {[...asks]
              .reverse()
              .slice(0, 5)
              .map((ask, index) => (
                <OrderBookRow key={index} color="red" price={ask.price} qty={ask.quantity} />
              ))}
            <tr>
              <td>
                <Money value={spread} />
              </td>
              <td style={{ textAlign: 'right' }}>
                <Money value={midPrice} />
              </td>
            </tr>
            {bids.slice(0, 5).map((bid, index) => (
              <OrderBookRow key={index} color="green" price={bid.price} qty={bid.quantity} />
            ))}
          </tbody>
        </table>
      </div>
      <Form midPrice={midPrice} />
    </div>
  );
}
