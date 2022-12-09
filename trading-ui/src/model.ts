export type Side = 'Buy' | 'Sell';

export type OrderStatus = 'Open' | 'Filled' | 'PartiallyFilled';

export interface Order {
  created_at: number;
  filled: string;
  id: number;
  price: string;
  quantity: string;
  side: Side;
  status: OrderStatus;
}
