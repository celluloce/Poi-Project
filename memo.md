# memo

## ToDo

> [ ] : 未解決
> [-] : 保留
> [x] : 解決済み

- actor
	- [x] 当たり判定を導入
		- [x] player, shot, enemyのコンストラクた書き換え
	- [x] Enemy当たり判定拡大
	- player 
		- [x] プレイヤを円で表示
		- [x] プレイヤを動かす
			- [x] キーインプットを入れる
			- [x] UpdatePoint関数を導入する
			- [x] なんか加速する問題を修正 
			- [x] Shiftで低速を導入
			- [x] 行動範囲を制限
				- [x] スクリーン幅を環境変数にする
	- shot
		- [x] コンストラクタ作る
		- [x] MainStateに組み込む
		- [x] MainStateのコンストラクタに組み込む 
		- [-] 実際に飛ばしてみる
			- [x] shotを長方形で描画
			- [x] ビームのようなshotになった
				- 問題がないと考えられるため保留
				- [x] MainStateにカウンタ導入
					- 上手く行かなかったのでボツ
			- [-] player_shot_timeoutなるものを導入
		- [x] 画面外に飛んだら消えるようにする
		- [ ] 弾間の隙間が大きすぎ問題
	- enemy
		- [x] コンストラクタ作成
		- [x] MainStateに組み込む
		- [x] 円として描写
		- [x] 時間計測を導入
			- [x] game counterを導入した
		- [x] PlayerShotとEnemyの当たり挙動を書く
		- [ ] 弾幕を張る
- feald
	- [ ] プレイ画面を作成
		- [ ] 色を変える
		- [ ] フィールドの値を定める
			- [ ] プレイヤの行動範囲を制限
	- [ ] プレイヤのライフを円で描写
- other
	- [ ] ややこしいのでy軸反転（現状では下がyが大きくなる）

## 当たり判定

## 時間計測
```
struct State {
	game_start_timer: Duration,
}

impl ggez::event::EventHandler for State {
	fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
		let start = get_time_since_start(ctx);
		println!("{:?}", start - self.game_start_timer);
		self.game_start_timer = start;
		
		Ok(())
	}
}
```

