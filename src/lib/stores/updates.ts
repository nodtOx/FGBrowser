import { writable } from 'svelte/store';

export interface UpdateStatus {
  isUpdating: boolean;
  message: string;
  newGamesFound: number;
}

export interface PopularStatus {
  isFetching: boolean;
  message: string;
  currentPeriod: string;
}

export const updateStatus = writable<UpdateStatus>({
  isUpdating: false,
  message: '',
  newGamesFound: 0,
});

export const popularStatus = writable<PopularStatus>({
  isFetching: false,
  message: '',
  currentPeriod: '',
});
