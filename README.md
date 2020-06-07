# StreamLogger

Logging utility for streamers.


## How to use

Streamers use this to summarize segements of their stream while streaming. It works as follows:

1. Stream starts. Streamer tell StreamLogger to start a new log.
2. Stream do stuff on stream. When stuff it done, they summarize the stuff in StreamLogger. Move on to next
   struff in their stream.

That's it.

## What it does

When StreamLogger is told to start a new log, it notes the time.

When StreamLogger is told to record a summary, it puts the summary and the previous noted time together. In
addition, it records the time again for the next summary.

The end result is the stream segment's summary is paired with the start time of the segment.

## Additional functionality

The log can be used to generate timestamps for the stream archive video. It is assumed that the start time of
the stream and the creation time of the log is diferent. User can look in the video archive to see how much
time has passed between the video and the time they started the log. Then, user can tell StreamLogger the
delta, and StreamLogger will use it to generate relative timestamps for the video archive for all the segment
summaries.
