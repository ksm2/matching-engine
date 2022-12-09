import { useEffect, useState } from 'react';
import { useInterval } from './useInterval';

export interface OrderBook {
  last: number | null;
  bestBid: number | null;
  bestAsk: number | null;
  bids: PricePoint[];
  asks: PricePoint[];
}

export interface PricePoint {
  price: number;
  quantity: number;
}

export function useOrderBook(): OrderBook {
  const [state, set] = useState<OrderBook>({
    last: null,
    bestBid: null,
    bestAsk: null,
    bids: [],
    asks: [],
  });

  useInterval(250, async () => {
    const response = await fetch('/api');
    const json = await response.json();
    set({
      last: toNumber(json.last),
      bestBid: toNumber(json.best_bid),
      bestAsk: toNumber(json.best_ask),
      bids: json.bids.map((pp: any) => ({ price: toNumber(pp.price)!, quantity: toNumber(pp.quantity)! })),
      asks: json.asks.map((pp: any) => ({ price: toNumber(pp.price)!, quantity: toNumber(pp.quantity)! })),
    });
  });

  return state;
}

function toNumber(value: string | null): number | null {
  return value === null ? null : Number(value);
}
