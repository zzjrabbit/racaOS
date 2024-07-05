# racaOS

![Logo](Logo.bmp)

## 简介
多次重构的系统，运行在x86_64平台 \
生日:2023-01-28 \
感谢[phil-opp](https://github.com/phil-opp)，他为我们提供了x86_64库 \
感谢[rCore团队](https://gitee.com/rcore-os)，他们为我们提供了引导模板、驱动模板以及中断重构思路 \
感谢[wenxuanjun](https://github.com/wenxuanjun)，他让racaOS进入了UEFI时代 \
raca_loader是基于bootloader改写的（当前基于bootloader V0.11.3）

### 目录
raca_core:主体 \
raca_loader:UEFI引导 \
esp:系统启动目录，将其中的所有文件复制粘贴到fat格式的U盘即可启动racaOS \
Docs:文档 \
.VSCodeCounter:代码行数统计

## 开发文档
开发文档请移步 \
[开发文档](Docs/index.md)


## TODO

- [x] raca_loader启动
- [x] 64位支持
- [ ] 英文显示
- [x] qemu xAPIC中断
- [x] 抢占式多任务
- [x] 定时器
- [x] 真机 xAPIC中断
- [x] 启动时间计算
- [ ] 密码登录
- [ ] 中文显示
- [ ] 完善的GUI
- [x] 文件系统
- [ ] 网卡驱动
- [ ] 声卡驱动
- [x] AHCI驱动
- [ ] 网络连接
- [x] 内存管理
- [ ] 完善的API接口
- [ ] 少量C库
- [ ] 图形库
