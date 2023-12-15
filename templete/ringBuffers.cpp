#include "ringBuffers.h"

ringBuffer*  ringBuffer_Out = new ringBuffer();
ringBuffer*  ringBuffer_In = new ringBuffer();

ringBuffer::ringBuffer() : readIndex(0), writeIndex(0), bytesCount(0)
{
    memset(buffer, 0, sizeof(buffer));
}

int nwrite(ringBuffer* rBuffer, const void* data, size_t num)
{

    // 计算实际要写入的数据数量，防止超过缓冲区的最大容量
    if (rBuffer->bytesCount == RING_BUFFER_SIZE)
        return 0;
    size_t writeCount = min(num, RING_BUFFER_SIZE - rBuffer->bytesCount);

    // 计算写入操作的内存大小
    size_t writeSize = writeCount ;

    // 计算拷贝的起始位置
    // size_t offset = writeIndex * sizeof(ZD_CAN_FRAME_T);

    // 执行批量拷贝操作
    if (writeSize <= (RING_BUFFER_SIZE - rBuffer->writeIndex) )
    {
        memcpy(&rBuffer->buffer[rBuffer->writeIndex], data, writeSize);
    }
    else {
        // 拷贝分为两部分：从写入位置到缓冲区末尾，以及从缓冲区起始位置到剩余数据末尾
        size_t firstPartSize = (RING_BUFFER_SIZE - rBuffer->writeIndex);
        size_t secondPartSize = writeSize - firstPartSize;

        memcpy(&rBuffer->buffer[rBuffer->writeIndex], data, firstPartSize);
        memcpy(rBuffer->buffer, reinterpret_cast<const char*>(data) + firstPartSize, secondPartSize);
    }

    // 更新写入索引和缓冲区中的数据计数
    rBuffer->writeIndex = (rBuffer->writeIndex + writeCount) % RING_BUFFER_SIZE;
    rBuffer->bytesCount += writeCount;

    // rBuffer->status = FREE;
    return writeCount;
}

int nread(ringBuffer* rBuffer, void* output, size_t num)
{

    // 计算实际要读取的数据数量，防止超过缓冲区中可用的数据数量
    size_t readCount = min(num, rBuffer->bytesCount);

    // 计算读取操作的内存大小
    size_t readSize = readCount ;
    // 执行批量读取操作
    if (readSize <= (RING_BUFFER_SIZE - rBuffer->readIndex) )
    {
        memcpy(output, &rBuffer->buffer[rBuffer->readIndex], readSize);
    }
    else {
        // 读取分为两部分：从读取位置到缓冲区末尾，以及从缓冲区起始位置到剩余数据末尾
        size_t firstPartSize = (RING_BUFFER_SIZE - rBuffer->readIndex) ;
        size_t secondPartSize = readSize - firstPartSize;
        memcpy(output, &rBuffer->buffer[rBuffer->readIndex], firstPartSize);
        memcpy(reinterpret_cast<char*>(output) + firstPartSize, rBuffer->buffer, secondPartSize);
    }

    // 更新读取索引和缓冲区中的数据计数
    rBuffer->readIndex = (rBuffer->readIndex + readCount) % RING_BUFFER_SIZE;
    rBuffer->bytesCount -= readCount;

    // rBuffer->status = FREE;
    return readCount;
}

int clear(ringBuffer* rBuffer)
{
    rBuffer->readIndex = 0;
    rBuffer->writeIndex = 0;
    rBuffer->bytesCount = 0;

    return 0;
}

size_t nreadable(ringBuffer* rBuffer)
{
    return rBuffer->bytesCount;
}


