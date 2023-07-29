#include "yahdlc/C/yahdlc.h"
#include <stdio.h>
#include <stdlib.h>

// As the instructions allow for a small bit of interpretation,
// I will describe my assumptions here.

// I interpreted the coordinate system to be the following
//  --- --- --- --- ---
// |0,0|   |   |   |4,0|
//  --- --- --- --- ---
// |   |   |   |   |   |
//  --- --- --- --- ---
// |   |   |   |   |   |
//  --- --- --- --- ---
// |   |   |   |   |   |
//  --- --- --- --- ---
// |0,4|   |   |   |4,4|
//  --- --- --- --- ---
//
// Position = (x, y)
// ↑ Up => y - 1
// ↓ Down => y + 1
// → Right => x + 1
// ← Left => x - 1

// "Leaving the board is an illegal move" => Any move that would have caused
// the character to leave the board is discarded, and the game proceeds.

// "If the same instruction occurs three times in a row, all three instructions
// should be discarded" => When three of the same type are found, they are immediately
// discarded, allowing a fourth and even fifth instruction of the same type to get through.
// A sixth will, of course, form a new run of 3 consecutive identical instructions, which will
// again result in their removal.


// An iterator inspired by the design from Rust. It recognizes sequences in data surrounded by
// YAHDLC_FLAG_SEQUENCE, and then feeds that sequence to the decoder. If the result is a data
// frame, the move is written to "output" and 1 is returned to indicate the iterator isn't
// empty yet. Once the iterator is empty, 0 is returned.
typedef struct{
  int start;
  int end;
  char *data;
  unsigned int data_len;
  char *output;
  unsigned int output_len;
} move_iterator_t;

// move_iterator_t.next()
int next_move(move_iterator_t* iter, yahdlc_control_t* control){
  while (iter->end < iter->data_len){
    
    if (iter->data[iter->end] == YAHDLC_FLAG_SEQUENCE){
      int ret = yahdlc_get_data(
        control,
        iter->data + iter->start,
        iter->end + 1 - iter->start,
        iter->output,
        &(iter->output_len)
      );
      if (ret < 0){
        printf("yahdlc error %i", ret);
        exit(1);
      }
      iter->start = iter->end + 1;
      iter->end += 2;
      if (control->frame != 0){
        continue;
      }
      return 1;
    } else{
      iter->end += 1;
    }
  }
  return 0;
}

typedef struct{
  int x;
  int y;
} player_position_t;

// A convenience function to make sure we don't exit the board
int clamp(int value, int min, int max) {
    if (value < min) {
        return min;
    } else if (value > max) {
        return max;
    } else {
        return value;
    }
}

void update_player(player_position_t* player, char move){
  switch (move){
    case 1: // Up
      player->y = clamp(player->y - 1, 0, 4);
      break;
    case 2: // Down
      player->y = clamp(player->y + 1, 0, 4);
      break;
    case 3: // Right
      player->x = clamp(player->x + 1, 0, 4);
      break;
    case 4: // Left
      player->x = clamp(player->x - 1, 0, 4);
      break;
  }
}

int main(){
  // The lack of include_bytes!() means that we have to dynamically load the file at runtime.
  // Good for memory footprint of the binary itself, bad for speed and safety.
  // Completely irrelevant to real life, as there the packets would be sent over some
  // connection.
  FILE* fileptr = fopen("../transmission.bin", "rb");
  if (!fileptr){
    printf("Could not find transmissions.bin\n");
  }

  fseek(fileptr, 0, SEEK_END);
  long filelen = ftell(fileptr);
  rewind(fileptr);
  
  char* buffer = malloc((filelen) * sizeof(char));
  char* output_buffer = malloc((filelen) * sizeof(char));
  
  fread(buffer, filelen, 1, fileptr);
  fclose(fileptr);

  move_iterator_t iter = {
    .start = 0,
    .end = 1,
    .data = buffer,
    .data_len = filelen,
    .output = output_buffer,
    .output_len = 0,
  };
  yahdlc_control_t control;

  int prev_idx = 0;

  // Start the player at (0, 4) according to the task.
  player_position_t player = {
    .x = 0,
    .y = 4,
  };
  
  // -1 is used to represent "no move"
  int prev_moves[] = {-1, -1, -1};
  
  int run;
  
  while (1){
    // Keep pulling moves until the iterator tells us there are no more
    run = next_move(&iter, &control);
    if (!run){
      break;
    }
    // Pop the oldest move and apply it if it exists
    char old_move = prev_moves[prev_idx];
    char move = iter.output[0];
    if (old_move != -1){
      update_player(&player, old_move);
    }
    prev_moves[prev_idx] = move;
    
    // Three in a row means clearing all the old moves
    int three_in_a_row = (prev_moves[0] == prev_moves[1]) && (prev_moves[0] == prev_moves[2]);
    if (three_in_a_row){
      prev_moves[0] = -1;
      prev_moves[1] = -1;
      prev_moves[2] = -1;
    }

    prev_idx = (prev_idx + 1) % 3;

    // Since the iterator doesn't explicitly return the data but merely modifies the output buffer, we are
    // responsible for resetting the length ourselves in this implementation.
    iter.output_len = 0;
  }

  // Get the rest of the old moves
  for (int i = 0; i < 3; i++){
    char move = prev_moves[prev_idx];
    if (move != -1){
      update_player(&player, move);
    }
    prev_idx = (prev_idx + 1) % 3;
  }

  // Report the answer
  printf("Player position is at (%i, %i)", player.x, player.y);
  
  // Always clean up after yourself
  free(buffer);
  free(output_buffer);
  
  return 0;
}
