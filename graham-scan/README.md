## Intro

This is an implementation of the [Graham Scan](https://en.wikipedia.org/wiki/Graham_scan) algorithm to find the [convex hull](https://en.wikipedia.org/wiki/Convex_hull) of a set of points. 

## The algorithm
The algorithm takes the following steps:
* Find the lowest point `base_point` (furthest left if there is a tie)
* Find the angle between the x axis and the vector `base_point`->`point_i` for each other `point_i` 
    * (It is not strictly neccessary to calculate the angle, any function of the angle that is monotonic in the range `[0,pi]` will do. In this implementation `cos` is used, and is calculated via the dot product.)
* Sort these points by this angle (such that they are ordered *counter-clockwise* with respect to `base_point`)
* Push `base_point` to a stack
* Push the first of the sorted points to the stack
* Repeat until all points have been pushed:
    * Push the next point in the sorted points to the stack
    * Consider the top 3 points in the stack
    * If they constitute a right hand turn (or are colinear), remove the middle of these three points from the stack, as it is causing a concavity
    * Repeat these last two steps until a left hand turn is encountered

In order to check for left or right hand turns the z component of the cross product of the vectors `P1->P2` and `P1->P3` can be used. Positive values for left turn, zero for colinear, and negative for right hand turn.

Once the angle for each point has been calculated, some might need to be culled. If 2 points and the base point are colinear, the nearer of the 2 points should be removed.

## Learned / Used

* File IO in rust
* Logging using tracing crate
* Basic Cargo stuff
* Test suites
* Separating out into modules
* Iterator and iterator methods
* Implementing traits for custom data types
