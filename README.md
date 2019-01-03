#### 功能: 迅雷链克口袋，链克商城抢购

由于本程序为本人个人使用，大多部分为个人习惯定制，并没有进行泛化，比如强制默认钱包密码为 123456789。放上来仅供大家学习参考。

**参数说明:**

- --balance <路径> 查询 wallets 目录下钱包余额，[路径] 参数可选填
- --collect 将 wallets 目录下所有子钱包内余额转至当前目录下主钱包内 (为防止错转造成损失，特设置成不能通过手动输入主钱包地址)
- --dispatch 将当前目录下主钱包的余额分发至 wallets 目录下各钱包 (过程中会要求输入分发数量)
- --export 批量导出链克商城交易成功后得到的兑换码
- --gen-wallet 批量生成钱包 (过程中会要求输入生成数量以及钱包密码)
- --redeem 按 conf.json 中的配置开始抢购 (兑换)
- --settle 抢购后结算，由于迅雷更新后不允许发送签名好的交易，商城内的交易按合约处理，目前无法使用
- --transact <发起转账钱包路径> <发起转账钱包密码> <接受转账地址> <gas limit> <data> 转账，[gas limit, data] 参数可选填

`collect`，`dispatch` 主要是用来防封号，多个账号交易被同一个钱包签名非常容易被封。
除 `transact` 以外，一律默认钱包密码为 **123456789**。

如报错请先检查目录下是否有主钱包文件 (必须 **0x** 开头以地址命名)，**accounts.txt** 用于存放抢购账号文件 (格式要求: 用户名=密码，例如 13333333333=ABC12345) 是否存在，**conf.json** (用于配置) 是否存在，**detectors.txt** (存放检测账号，可在配置文件中开启检测功能) 是否存在。

conf.json 说明:

```json
{
    "proxy_pool_api": "",
    "transaction_proxy": "",
    "date": "",
    "request_timeout": 1,
    "account_per_thread": 1,
    "wallet_per_thread": 1,
    "transaction_per_thread": 1,
    "detect": true,
    "export_with_proxy": true,
    "redeem_with_proxy": true,
    "kinds": [
        10
    ]
}
```

- `proxy_pool_api`: ip 池地址，返回格式需为文本格式，抢购必备不解释
- `transaction_proxy`: 交易代理地址，一般填写 ss 代理地址，格式 `http://127.0.0.1:1080`，因为交易需要翻墙
- `date`: 日期，格式 20181126
- `request_timeout`: 请求超时时间，由于代理池质量各不相同，设置超时上线
- `account_per_thread`: 每个线程分配几个账号，例如 accounts.txt 中有 300 个账号，此处填写 5 那么就是 60 线程
- `wallet_per_thread`: 每个线程分配几个支付钱包，由于 settle 功能目前无法使用此项无用，但必须填写
- `detect`: 自动检测是否有货，知道准点放货的时候推荐不开会影响抢购速度，有时候商城会偷偷上货所以有此功能
- `export_with_proxy`: 导出兑换码的时候是否启用代理，推荐启用否则容易出现操作频繁
- `redeem_with_proxy`: 兑换时是否启用代理，推荐启用理由同上
- `kinds`: 抢购商品详见下面

 1. 爱奇艺黄金会员12个月
 2. 爱奇艺VIP钻石会员年卡
 3. 爱奇艺黄金会员6个月
 4. 爱奇艺会员季卡
 5. 爱奇艺会员月卡
 6. 爱奇艺黄金会员周卡
 7. 爱奇艺钻石VIP会员
 8. 迅雷超级会员月卡
 9. 迅雷白金会员月卡
 10. 《链与消消乐》邀请码
 
 想添加其他商品自行上 app 抓包修改代码。另外抢购推荐提前几分钟开启，某些代理 ip 速度那是相当慢。
 
 本人实际使用效果 10 秒能抢 300 张卡左右，最大影响因素为 ip 池质量。