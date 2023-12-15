#pragma once
#ifndef _RINGBUFFERS_H
#define _RINGBUFFERS_H

#include "protocol.h"

class ringBuffer {
public:
    size_t readIndex;             
    size_t writeIndex;                 
    size_t bytesCount;
    uint8_t buffer[RING_BUFFER_SIZE];

public:
    ringBuffer();
};


int nwrite(ringBuffer* rBuffer, const void* data, size_t num);
int nread(ringBuffer* rBuffer, void* output, size_t num);
int clear(ringBuffer* rBuffer);
size_t nreadable(ringBuffer* rBuffer);


#endif