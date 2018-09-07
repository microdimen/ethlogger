pragma solidity ^0.4.0;

contract simple {
    event Count(uint indexed counter);

    uint counter;

    constructor ()
    public
    {
        counter = 0;
    }

    function increase()
    public
    {
        counter += 1;
        emit Count(counter);
    }
}
