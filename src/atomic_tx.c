#include <pthread.h>
#include <stdint.h>
#include <stdio.h>

typedef struct {
  uint32_t sender;
  uint32_t receiver;
} tx_data;

typedef struct {
  tx_data data;
  pthread_mutex_t mutex;
} tx_state;

uint32_t tx_fees(uint32_t amount) {
  if (amount <= 10)
    return 2;
  else if (amount <= 100)
    return 5;
  else if (amount <= 500)
    return 10;
  else
    return amount / 50;
}

int atomic_transfer_1(tx_state *state, uint32_t amount) {
  int ret = 0;
  uint32_t fees = tx_fees(amount);

  pthread_mutex_lock(&state->mutex);
  tx_data *data = &state->data;

  if (data->sender < fees) {
    ret = -1;
    goto EXIT;
  }
  data->sender -= fees;

  if (data->sender < amount) {
    ret = -2;
    goto EXIT_1;
  }
  data->sender -= amount;

  data->receiver += amount;
  goto EXIT;

EXIT_1:
  data->sender += fees;
EXIT:
  pthread_mutex_unlock(&state->mutex);
  return ret;
}

int atomic_transfer_2(tx_state *state, uint32_t amount) {
  int ret = 0;
  uint32_t fees = tx_fees(amount);

  pthread_mutex_lock(&state->mutex);
  tx_data *data = &state->data;

  if (data->sender >= fees) {
    data->sender -= fees;
    if (data->sender >= amount) {
      data->sender -= amount;
      data->receiver += amount;
    } else {
      ret = -2;
      data->sender += fees;
    }
  } else {
    ret = -1;
  }

  pthread_mutex_unlock(&state->mutex);
  return ret;
}

int main() {
  tx_state state = {.data = {.sender = 10, .receiver = 20},
                    .mutex = PTHREAD_MUTEX_INITIALIZER};
  int ret = atomic_transfer_1(&state, 8);
  printf("Returned: %d\n", ret);
  printf("{sender: %u, receiver: %u}\n", state.data.sender,
         state.data.receiver);
  
  atomic_transfer_2(&state, 8);
  printf("Returned: %d\n", ret);
  printf("{sender: %u, receiver: %u}\n", state.data.sender,
         state.data.receiver);
  return 0;
}