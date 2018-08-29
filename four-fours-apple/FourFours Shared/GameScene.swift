//
//  GameScene.swift
//  FourFours Shared
//
//  Created by Kevin Guthrie on 8/9/18.
//  Copyright © 2018 Rook And Pawn Industries, Inc. All rights reserved.
//

import SpriteKit

class GameScene: SKScene {
    
    
  class func newGameScene(size: CGSize) -> GameScene {
    // Load 'GameScene.sks' as an SKScene.
    guard let scene = SKScene(fileNamed: "Empty.sks") as? GameScene else {
      print("Failed to load GameScene.sks")
      abort()
    }
  
    // Set the scale mode to scale to fit the window
    scene.size = size
    scene.scaleMode = .aspectFill
    
    scene.anchorPoint = CGPoint(x: 0.0, y: 1.0)
    
    return scene
  }

  private var currentView : BaseView?
  
  func setUpScene() {
    let systemView = SystemView(textureLoader: TextureLoader())

    let ctx = RustBinder.bindToRust(systemView)
    
    let transitioner = TransitionService(transitionClosure: { (view) in
      DispatchQueue.main.async {
        self.removeAllChildren()
        self.removeAllActions()
        self.addChild(view)
        self.currentView = view
        self.setSize(size: self.size)
      }
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
  
  func setSize(size: CGSize) {
    DispatchQueue.main.async {
      self.size = size
      self.currentView?.setSize(size: self.size)
    }
  }
  
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

