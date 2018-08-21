//
//  GameScene.swift
//  FourFours Shared
//
//  Created by Kevin Guthrie on 8/9/18.
//  Copyright © 2018 Rook And Pawn Industries, Inc. All rights reserved.
//

import SpriteKit

class GameScene: SKScene {
    
    
  class func newGameScene() -> GameScene {
    // Load 'GameScene.sks' as an SKScene.
    guard let scene = SKScene(fileNamed: "Empty") as? GameScene else {
      print("Failed to load GameScene.sks")
      abort()
    }
  
    // Set the scale mode to scale to fit the window
    scene.scaleMode = .aspectFill
  
    return scene
  }

  private var currentView : BaseView<Any>?
  
  func setUpScene() {
    let systemView = SystemView(textureLoader: TextureLoader())

    let ctx = RustBinder.bindToRust(systemView)
    
    let transitioner = TransitionService(transitionClosure: { (view) in
      self.removeAllChildren()
      self.removeAllActions()
      self.addChild(view)
    })
    
    let loadingView = LoadingView()
    
    loadingView.initializeCtx(ctx: ctx, transitionService: transitioner)
    transitioner.transition(view: loadingView)
    loadingView.setPresenter(presenter: ctx.bindToLoadingView(view: loadingView))
    
    
  }
  
  #if os(watchOS)
  override func sceneDidLoad() {
    self.setUpScene()
  }
  #else
  override func didMove(to view: SKView) {
    self.setUpScene()
  }
  #endif
  
  override func update(_ currentTime: TimeInterval) {
    // Called before each frame is rendered
  }
}

#if os(iOS) || os(tvOS)
// Touch-based event handling
extension GameScene {

  override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
    
  }
  
  override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
    
  }
  
  override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
    
  }
  
  override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
    
  }
    
   
}
#endif

#if os(OSX)
// Mouse-based event handling
extension GameScene {

  override func mouseDown(with event: NSEvent) {
    
  }
  
  override func mouseDragged(with event: NSEvent) {
    
  }
  
  override func mouseUp(with event: NSEvent) {
    
  }

}
#endif

