import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';
import { fetcher } from '..';
import { LoadingState } from '../rest-model';

export const fetchError = createAsyncThunk('errors/fetch', async (page: number, thunkApi) => {
  const ret = await fetcher(`/api/errors?page=${page}`);
  return ret;
});

interface ErrorState {
  total_number: number;
  total_page: number;
  items: any[];
  status: LoadingState;
}

const initialState: ErrorState = {
  total_number: 0,
  total_page: 1,
  items: [],
  status: LoadingState.NotReady,
};

export const errorsSlice = createSlice({
  name: 'errors',
  initialState,
  reducers: {},
  extraReducers: (builder) => {
    builder.addCase(fetchError.pending, (state, action) => {
      state.status = LoadingState.Loading;
    });

    builder.addCase(fetchError.fulfilled, (state, action) => {
      state.status = LoadingState.Success;
      state.total_number = action.payload.total_count;
      state.total_page = action.payload.total_page;
      state.items = action.payload.records;
    });
  },
});
