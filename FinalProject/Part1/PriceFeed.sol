// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "@chainlink/contracts/src/v0.8/interfaces/AggregatorV3Interface.sol";
import "./interfaces/IPriceFeed.sol";

contract PriceFeed is IPriceFeed {

    AggregatorV3Interface internal priceFeed;

    constructor(address proxy) {
        priceFeed = AggregatorV3Interface(proxy);
    }

    function getLatestPrice() override external view returns (
        int price,
        uint lastUpdatedTime
    ) {
        ( , int256 answer, , uint256 updatedAt, ) =  priceFeed.latestRoundData();
        price = answer;
        lastUpdatedTime = updatedAt;
    }
}