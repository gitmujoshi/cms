import os
import tensorflow as tf
import boto3
from datetime import datetime

# Get environment variables
MODEL_CHECKPOINT_PATH = os.environ.get('MODEL_CHECKPOINT_PATH', 's3://contract-management-bucket-bkvww1ps/checkpoints')
EPOCHS = int(os.environ.get('EPOCHS', '10'))
BATCH_SIZE = int(os.environ.get('BATCH_SIZE', '32'))

# Load MNIST dataset
print("Loading MNIST dataset...")
(x_train, y_train), (x_test, y_test) = tf.keras.datasets.mnist.load_data()
x_train = x_train.astype('float32') / 255.0
x_test = x_test.astype('float32') / 255.0

# Create model
print("Creating model...")
model = tf.keras.Sequential([
    tf.keras.layers.Flatten(input_shape=(28, 28)),
    tf.keras.layers.Dense(128, activation='relu'),
    tf.keras.layers.Dropout(0.2),
    tf.keras.layers.Dense(10, activation='softmax')
])

# Compile model
model.compile(optimizer='adam',
              loss='sparse_categorical_crossentropy',
              metrics=['accuracy'])

# Create checkpoint callback
checkpoint_path = f"{MODEL_CHECKPOINT_PATH}/mnist_model_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
checkpoint_callback = tf.keras.callbacks.ModelCheckpoint(
    filepath=checkpoint_path,
    save_weights_only=True,
    save_best_only=True,
    monitor='val_accuracy',
    mode='max',
    verbose=1
)

# Train model
print(f"Starting training for {EPOCHS} epochs with batch size {BATCH_SIZE}...")
history = model.fit(
    x_train, y_train,
    epochs=EPOCHS,
    batch_size=BATCH_SIZE,
    validation_split=0.2,
    callbacks=[checkpoint_callback]
)

# Evaluate model
print("Evaluating model...")
test_loss, test_accuracy = model.evaluate(x_test, y_test, verbose=2)
print(f"\nTest accuracy: {test_accuracy:.4f}")

# Save final model
final_model_path = f"{checkpoint_path}_final"
model.save(final_model_path)
print(f"Model saved to {final_model_path}") 