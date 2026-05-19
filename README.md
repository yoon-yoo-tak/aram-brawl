# aram-brawl

> 무작위 총력전 (ARAM) 아수라장 모드를 CLI에서 즐기는 작은 게임.
> 챔피언이 무작위로 정해지고, 4라운드에 걸쳐 증강을 골라 빌드를 완성한다.

## 설치

### Homebrew (macOS / Linux)

```bash
brew tap yoon-yoo-tak/tap
brew install aram-brawl
```

업데이트:
```bash
brew upgrade aram-brawl
```

### Cargo (소스에서 직접 빌드)

```bash
cargo install --git https://github.com/yoon-yoo-tak/aram-brawl
```

## 실행

```bash
aram-brawl
```

조작:
- `1`, `2`, `3` — 카드 선택
- `r1`, `r2`, `r3` — 해당 슬롯 리롤 (라운드당 슬롯별 1회)
- `q` — 종료

## 개발

```bash
cargo run --release
```

## License

MIT
