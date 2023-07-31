# Designs

## Discussion on web framework
`actix-web` was chosen to be the web framework for the API. `actix-web` is a well-known high performance web framework, can be setup easily, well-maintained and have many features that can be used to help the development, e.g. setting up an log middleware for debugging in manual testing. It also have many library support which helps to reduce development time.

## Discussion on rate limiter
`actix-governor` is chosen as the rate limiting middleware in the project. Under the hood, it is using `governor` as the implemenation of the rate limiter. It is a popular rate limiting library, and it uses an variant of Leaky Bucket Algorithm (GCRA) to implement the rate limiting.

An existing rate limiting library is chosen as this could save time for implementing rate limiting algorithms, and the library itself is well-maintained and offers a good amount of customization that could be used, e.g. to implement the key extraction and error response for the keyed rate limiter middleware.

This have an important assumption that the requirement is given in an average sense: the request should rate limit in average e.g. 1200 request per minute, although at times it could exceed the limit if the quota is not all used.

It works by having a quota and a replenishing time. Say if the quota is 1200 requests, and the repleishing time for one quota is 50ms, if the quota is not full, it will replenish quota 50ms after the quota is used. Thus, if you do 1200 request at once, you will need to wait 50ms to do another, or you will need to wait 1 minute for doing another full 1200 burst of requests. This could potentially allow a time window of 1 minute having more than 1200 requests accepted, but for a period of time, it averages to <1200 requests/min.

`actix-governor` is basically an `actix-web` middleware wrapper for `governor` that implements all the middleware bridging, and implements some useful configurations, such as adding `x-ratelimit-remaining`, `x-ratelimit-limit` and `x-ratelimit-after` headers and configuring with custom key extractor. This can be useful for large project to provide flexibility on various use cases of the rate limiter. The quota / replenish time for the rate limiter can be easily set through the library.

## Discussion on testing the rate limiter
Different approaches were used to test the rate limiter in different rate limiting setting.

For the 3 req/min endpoint, it is tested by simulating responses on doing 4 continuous requests and to see if the output is normal (i.e. final request is being rate limited). A 60s timeout is also used to see if the rate limiter replenish and next burst of 3 can be sent. A 20s timeout + 1 request loop is used to test the behaviour of seperating the traffic instead of bursting.

For the 1200 req/min endpoint, it is tested differently as the endpoint replenish fast and it is hard to make the execution time of the test case to provide a constant timing in ms. Instead, a burst of 1300 request is done to see how many of those requests passes, and the test allows a small amount of offset to compensate the timing used during the test. Another burst is also tested after 60s of timeout.

For the per path endpoint, it is similar to the 3 req/min endpoint but have extra test to ensure the behaviour of rate limiting different path and make sure that works independently.
